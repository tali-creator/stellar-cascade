//! Thin wrapper around [`stellar_rpc_client::Client`] for fetching contract
//! events from Soroban RPC.
//!
//! The rest of the codebase should import this module rather than calling
//! `stellar_rpc_client` directly, so the RPC boundary is in one place and
//! easy to mock or swap later.

use base64::Engine as _;
use stellar_rpc_client::{Client, Event, EventStart, EventType};
use stellar_xdr::{
    ContractDataDurability, ContractId, Hash, LedgerKey, LedgerKeyContractData, Limits, ReadXdr,
    ScAddress, ScBytes, ScSymbol, ScVal, ScVec, StringM, VecM, WriteXdr,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("failed to create RPC client: {0}")]
    ClientInit(#[from] Box<stellar_rpc_client::Error>),

    #[error("getEvents RPC call failed: {0}")]
    GetEvents(Box<stellar_rpc_client::Error>),

    #[error("getLedgerEntries RPC call failed: {0}")]
    GetLedgerEntries(Box<stellar_rpc_client::Error>),

    #[error("XDR encode/decode error: {0}")]
    Xdr(#[from] stellar_xdr::Error),

    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("invalid contract ID '{0}': {1}")]
    InvalidContractId(String, String),
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Re-export so callers don't need to depend on `stellar-rpc-client` directly.
pub type RawEvent = Event;

/// A receiver as read back from on-chain contract storage.
#[derive(Debug, Clone)]
pub struct ReceiverOnChain {
    pub address: String,
    pub percentage_bps: u32,
}

/// A project as read back from on-chain contract storage.
#[derive(Debug, Clone)]
pub struct ProjectOnChain {
    #[allow(dead_code)] // owner is written to DB via apply_register; field kept for completeness
    pub owner_address: String,
    pub receivers: Vec<ReceiverOnChain>,
}

// ---------------------------------------------------------------------------
// get_events
// ---------------------------------------------------------------------------

/// Fetch contract events from Soroban RPC starting at `start_ledger`,
/// filtered to `contract_id`.
///
/// Returns up to `limit` events (default 100 if `None`).  The caller is
/// responsible for paginating by advancing `start_ledger` to
/// `last_event.ledger + 1` after each batch.
pub async fn get_events(
    rpc_url: &str,
    contract_id: &str,
    start_ledger: u32,
    limit: Option<usize>,
) -> Result<Vec<RawEvent>, RpcError> {
    let client = Client::new(rpc_url).map_err(Box::new)?;

    let response = client
        .get_events(
            EventStart::Ledger(start_ledger),
            Some(EventType::Contract),
            &[contract_id.to_string()],
            &[], // no topic filter — fetch all event types
            Some(limit.unwrap_or(100)),
        )
        .await
        .map_err(|e| RpcError::GetEvents(Box::new(e)))?;

    Ok(response.events)
}

// ---------------------------------------------------------------------------
// fetch_project
// ---------------------------------------------------------------------------

/// Fetch the current `Project` struct from contract persistent storage.
///
/// The `DataKey::Project(BytesN<32>)` Soroban contracttype enum serialises to
/// `ScVal::Vec([ScVal::Symbol("Project"), ScVal::Bytes(id_bytes)])`.
///
/// Returns `None` if the entry does not exist or has expired.
pub async fn fetch_project(
    rpc_url: &str,
    contract_id: &str,
    project_id_hex: &str,
) -> Result<Option<ProjectOnChain>, RpcError> {
    // Decode hex project_id → 32 bytes.
    let id_bytes = hex::decode(project_id_hex)
        .map_err(|e| RpcError::InvalidContractId(project_id_hex.to_string(), e.to_string()))?;
    if id_bytes.len() != 32 {
        return Err(RpcError::InvalidContractId(
            project_id_hex.to_string(),
            format!("expected 32 bytes, got {}", id_bytes.len()),
        ));
    }

    // Decode the C... contract strkey → 32-byte hash.
    let contract_hash = stellar_strkey::Contract::from_string(contract_id)
        .map_err(|e| RpcError::InvalidContractId(contract_id.to_string(), e.to_string()))?;

    // Build DataKey::Project(id) as ScVal::Vec([Symbol("Project"), Bytes(id)]).
    let data_key = ScVal::Vec(Some(ScVec(VecM::try_from(vec![
        ScVal::Symbol(ScSymbol(StringM::try_from("Project")?)),
        ScVal::Bytes(ScBytes(id_bytes.clone().try_into()?)),
    ])?)));

    // Build the LedgerKey.
    let ledger_key = LedgerKey::ContractData(LedgerKeyContractData {
        contract: ScAddress::Contract(ContractId(Hash(contract_hash.0))),
        key: data_key,
        durability: ContractDataDurability::Persistent,
    });

    // Serialise to base64 XDR for the RPC call.
    let key_xdr = ledger_key.to_xdr(Limits::none())?;
    let key_b64 = base64::engine::general_purpose::STANDARD.encode(&key_xdr);

    let client = Client::new(rpc_url).map_err(Box::new)?;
    let response = client
        .get_ledger_entries(&[LedgerKey::from_xdr(
            base64::engine::general_purpose::STANDARD.decode(&key_b64)?,
            Limits::none(),
        )?])
        .await
        .map_err(|e| RpcError::GetLedgerEntries(Box::new(e)))?;

    let entries = match response.entries {
        Some(ref e) if !e.is_empty() => e,
        _ => return Ok(None),
    };

    // Decode the returned XDR value.
    let entry_xdr = base64::engine::general_purpose::STANDARD.decode(&entries[0].xdr)?;

    // The value is a LedgerEntry; extract the contract data value (ScVal).
    use stellar_xdr::{LedgerEntry, LedgerEntryData};
    let ledger_entry = LedgerEntry::from_xdr(entry_xdr, Limits::none())?;

    let project_val = match ledger_entry.data {
        LedgerEntryData::ContractData(d) => d.val,
        _ => return Ok(None),
    };

    // The Project struct serialises as:
    //   ScVal::Map([
    //     (Symbol("id"),        Bytes(32)),
    //     (Symbol("owner"),     Address),
    //     (Symbol("receivers"), Vec([Map([Symbol("address"), Address], [Symbol("percentage"), U32]), ...])),
    //   ])
    decode_project_scval(project_val).map(Some)
}

/// Decode a `ScVal::Map` representing a `Project` contracttype struct.
fn decode_project_scval(val: ScVal) -> Result<ProjectOnChain, RpcError> {
    use stellar_xdr::ScMap;

    let map = match val {
        ScVal::Map(Some(m)) => m,
        _ => {
            return Err(RpcError::Xdr(stellar_xdr::Error::Invalid));
        }
    };

    let get_field = |map: &ScMap, name: &str| -> Option<ScVal> {
        map.iter().find_map(|entry| {
            if matches!(&entry.key, ScVal::Symbol(s) if s.to_utf8_string_lossy() == name) {
                Some(entry.val.clone())
            } else {
                None
            }
        })
    };

    // owner field
    let owner_val = get_field(&map, "owner").ok_or(stellar_xdr::Error::Invalid)?;
    let owner_address =
        crate::event_decode::scval_to_strkey(owner_val).ok_or(stellar_xdr::Error::Invalid)?;

    // receivers field
    let receivers_val = get_field(&map, "receivers").ok_or(stellar_xdr::Error::Invalid)?;
    let receivers = decode_receivers_scval(receivers_val)?;

    Ok(ProjectOnChain {
        owner_address,
        receivers,
    })
}

/// Decode `ScVal::Vec([ReceiverMap, ...])` into a list of [`ReceiverOnChain`].
fn decode_receivers_scval(val: ScVal) -> Result<Vec<ReceiverOnChain>, RpcError> {
    let vec = match val {
        ScVal::Vec(Some(v)) => v,
        _ => return Err(RpcError::Xdr(stellar_xdr::Error::Invalid)),
    };

    vec.iter()
        .map(|item| {
            let map = match item {
                ScVal::Map(Some(m)) => m.clone(),
                _ => return Err(RpcError::Xdr(stellar_xdr::Error::Invalid)),
            };

            let get = |name: &str| -> Option<ScVal> {
                map.iter().find_map(|e| {
                    if matches!(&e.key, ScVal::Symbol(s) if s.to_utf8_string_lossy() == name) {
                        Some(e.val.clone())
                    } else {
                        None
                    }
                })
            };

            let address_val = get("address").ok_or(stellar_xdr::Error::Invalid)?;
            let address = crate::event_decode::scval_to_strkey(address_val)
                .ok_or(stellar_xdr::Error::Invalid)?;

            let percentage_bps = match get("percentage").ok_or(stellar_xdr::Error::Invalid)? {
                ScVal::U32(n) => n,
                _ => return Err(RpcError::Xdr(stellar_xdr::Error::Invalid)),
            };

            Ok(ReceiverOnChain {
                address,
                percentage_bps,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access to Stellar testnet"]
    async fn live_testnet_get_events_returns_register_event() {
        const RPC_URL: &str = "https://soroban-testnet.stellar.org:443";
        const CONTRACT_ID: &str = "CC4BP273CO73T6AXOMILHWXF37EJ5B766JEOKCYTET4YBHE3FX46GYNI";
        const START_LEDGER: u32 = 1;

        let events = get_events(RPC_URL, CONTRACT_ID, START_LEDGER, Some(50))
            .await
            .expect("getEvents should succeed against live testnet");

        for e in &events {
            println!("event: id={} topic={:?} value={}", e.id, e.topic, e.value);
        }

        // Contract has no registered projects yet — events list may be empty.
        // This test confirms the RPC call succeeds without panicking.
        println!("total events: {}", events.len());
    }
}
