//! XDR event decoding for registry contract events.
//!
//! The Soroban RPC returns events with `topic` (a `Vec<String>` of
//! base64-encoded `ScVal`s) and `value` (a single base64-encoded `ScVal`).
//!
//! Contract event shapes (from contracts/registry/src/lib.rs):
//!
//! ```text
//! register_project:
//!   topics : [ScSymbol("register"), ScBytes(project_id: BytesN<32>)]
//!   value  : ScVec([ScAddress(owner), ScU32(receiver_count)])
//!
//! update_splits:
//!   topics : [ScSymbol("update"), ScBytes(project_id: BytesN<32>)]
//!   value  : ScU32(new_receiver_count)
//!
//! deposit:
//!   topics : [ScSymbol("deposit"), ScBytes(project_id: BytesN<32>), ScAddress(token_address)]
//!   value  : ScI128(amount)
//!
//! The `deposit` event shape is defined for the future deposit-phase contract.
//! The decoder is implemented here so `apply_event` can handle it as soon as
//! the contract is deployed; until then it will never appear on-chain.
//! ```

#![allow(dead_code)] // used by the sync worker in a later issue

use base64::Engine as _;
use stellar_xdr::{Limits, ReadXdr, ScAddress, ScVal};
use thiserror::Error;

use crate::soroban_rpc::RawEvent;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("base64 decode failed for field '{field}': {source}")]
    Base64 {
        field: &'static str,
        #[source]
        source: base64::DecodeError,
    },

    #[error("XDR decode failed for field '{field}': {source}")]
    Xdr {
        field: &'static str,
        #[source]
        source: stellar_xdr::Error,
    },

    #[error("event has {actual} topics, expected at least 2")]
    TooFewTopics { actual: usize },

    #[error("first topic is not a Symbol: {0:?}")]
    NotASymbol(ScVal),

    #[error("unknown event type symbol: '{0}'")]
    UnknownEventType(String),

    #[error("unexpected ScVal shape for project_id topic: {0:?}")]
    BadProjectId(ScVal),

    #[error("unexpected ScVal shape for register value: {0:?}")]
    BadRegisterValue(ScVal),

    #[error("unexpected ScVal shape for update value: {0:?}")]
    BadUpdateValue(ScVal),

    #[error("unexpected ScVal shape for deposit topic/value: {0:?}")]
    BadDepositValue(ScVal),
}

// ---------------------------------------------------------------------------
// Decoded event types
// ---------------------------------------------------------------------------

/// A fully decoded registry contract event.
#[derive(Debug, Clone, PartialEq)]
pub enum DecodedEvent {
    /// Emitted by `register_project`.
    ///
    /// Contract topics:  `[Symbol("register"), BytesN<32>(project_id)]`
    /// Contract value:   `(Address(owner), u32(receiver_count))`
    RegisterProject {
        /// 32-byte project ID as lowercase hex (matches the `projects.id` column).
        project_id: String,
        /// Stellar G... address of the project owner.
        owner_address: String,
        /// Number of receivers declared at registration time.
        receiver_count: u32,
    },

    /// Emitted by `update_splits`.
    ///
    /// Contract topics:  `[Symbol("update"), BytesN<32>(project_id)]`
    /// Contract value:   `u32(new_receiver_count)`
    UpdateSplits {
        /// 32-byte project ID as lowercase hex.
        project_id: String,
        /// Number of receivers in the new split configuration.
        new_receiver_count: u32,
    },

    /// Emitted by the deposit-phase contract when funds are deposited.
    ///
    /// Contract topics:  `[Symbol("deposit"), BytesN<32>(project_id), Address(token_address)]`
    /// Contract value:   `i128(amount)`
    ///
    /// The deposit is additive: the sync worker accumulates amounts into the
    /// `balances` table rather than overwriting them.
    Deposit {
        /// 32-byte project ID as lowercase hex.
        project_id: String,
        /// Stellar contract address (C... strkey) of the deposited token.
        token_address: String,
        /// Amount in the token's smallest unit (i128, stored as NUMERIC in Postgres).
        amount: i128,
    },
}

// ---------------------------------------------------------------------------
// Decode helpers
// ---------------------------------------------------------------------------

/// Decode a single base64-encoded XDR `ScVal`.
fn decode_scval(b64: &str, field: &'static str) -> Result<ScVal, DecodeError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| DecodeError::Base64 { field, source: e })?;

    ScVal::from_xdr(bytes, Limits::none()).map_err(|e| DecodeError::Xdr { field, source: e })
}

/// Extract the 32-byte project ID from `ScVal::Bytes` and hex-encode it.
fn project_id_from_scval(val: ScVal) -> Result<String, DecodeError> {
    match val {
        ScVal::Bytes(b) => Ok(hex::encode(b.as_slice())),
        other => Err(DecodeError::BadProjectId(other)),
    }
}

/// Convert an `ScVal::Address` to a Stellar strkey string.
/// Exported for use by `soroban_rpc::decode_project_scval`.
pub fn scval_to_strkey(val: ScVal) -> Option<String> {
    match val {
        ScVal::Address(addr) => match addr {
            ScAddress::Account(account_id) => {
                use stellar_xdr::PublicKey;
                match account_id.0 {
                    PublicKey::PublicKeyTypeEd25519(key) => {
                        Some(stellar_strkey::ed25519::PublicKey(key.0).to_string())
                    }
                }
            }
            ScAddress::Contract(hash) => Some(stellar_strkey::Contract(hash.0.0).to_string()),
            // MuxedAccount, ClaimableBalance, LiquidityPool are not expected
            // in registry contract events; return None so the caller can error.
            _ => None,
        },
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Public decode entry point
// ---------------------------------------------------------------------------

/// Decode a raw RPC event into a typed [`DecodedEvent`].
///
/// Returns `None` for events with unrecognised topic symbols — these are
/// silently skipped by the sync worker rather than causing a fatal error.
pub fn decode_event(raw: &RawEvent) -> Result<Option<DecodedEvent>, DecodeError> {
    // Need at least [symbol, project_id] in topics.
    if raw.topic.len() < 2 {
        return Err(DecodeError::TooFewTopics {
            actual: raw.topic.len(),
        });
    }

    // Decode the first topic — must be a Symbol.
    let topic0 = decode_scval(&raw.topic[0], "topic[0]")?;
    let event_symbol = match &topic0 {
        ScVal::Symbol(s) => s.to_utf8_string_lossy(),
        other => return Err(DecodeError::NotASymbol(other.clone())),
    };

    // Decode the second topic — the project_id (BytesN<32> → ScVal::Bytes).
    let topic1 = decode_scval(&raw.topic[1], "topic[1]")?;
    let project_id = project_id_from_scval(topic1)?;

    match event_symbol.as_str() {
        "register" => {
            // value: ScVec([ScAddress(owner), ScU32(receiver_count)])
            let value = decode_scval(&raw.value, "value")?;
            let (owner_val, count_val) = match value {
                ScVal::Vec(Some(ref vec)) if vec.len() == 2 => (vec[0].clone(), vec[1].clone()),
                other => return Err(DecodeError::BadRegisterValue(other)),
            };

            let owner_address = scval_to_strkey(owner_val.clone())
                .ok_or(DecodeError::BadRegisterValue(owner_val))?;

            let receiver_count = match count_val {
                ScVal::U32(n) => n,
                other => return Err(DecodeError::BadRegisterValue(other)),
            };

            Ok(Some(DecodedEvent::RegisterProject {
                project_id,
                owner_address,
                receiver_count,
            }))
        }

        "update" => {
            // value: ScU32(new_receiver_count)
            let value = decode_scval(&raw.value, "value")?;
            let new_receiver_count = match value {
                ScVal::U32(n) => n,
                other => return Err(DecodeError::BadUpdateValue(other)),
            };

            Ok(Some(DecodedEvent::UpdateSplits {
                project_id,
                new_receiver_count,
            }))
        }

        "deposit" => {
            // topics: [Symbol("deposit"), BytesN<32>(project_id), Address(token_address)]
            // value:  ScI128(amount)
            //
            // Requires a third topic for the token address.
            if raw.topic.len() < 3 {
                return Err(DecodeError::TooFewTopics {
                    actual: raw.topic.len(),
                });
            }

            let topic2 = decode_scval(&raw.topic[2], "topic[2]")?;
            let token_address =
                scval_to_strkey(topic2.clone()).ok_or(DecodeError::BadDepositValue(topic2))?;

            let value = decode_scval(&raw.value, "value")?;
            let amount = match value {
                ScVal::I128(parts) => {
                    // ScVal::I128 is an Int128Parts { hi: i64, lo: u64 }.
                    // Reconstruct the i128.
                    ((parts.hi as i128) << 64) | (parts.lo as i128)
                }
                other => return Err(DecodeError::BadDepositValue(other)),
            };

            Ok(Some(DecodedEvent::Deposit {
                project_id,
                token_address,
                amount,
            }))
        }

        other => Err(DecodeError::UnknownEventType(other.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soroban_rpc::RawEvent;
    use stellar_xdr::{
        AccountId, Limits, PublicKey, ScBytes, ScSymbol, ScVal, ScVec, StringM, Uint256, VecM,
        WriteXdr,
    };

    // -----------------------------------------------------------------------
    // Helpers to build real base64-encoded XDR fixtures
    // -----------------------------------------------------------------------

    fn encode_scval(val: &ScVal) -> String {
        let bytes = val.to_xdr(Limits::none()).unwrap();
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    fn symbol_val(s: &str) -> ScVal {
        ScVal::Symbol(ScSymbol(StringM::try_from(s).unwrap()))
    }

    fn bytes32_val(seed: u8) -> ScVal {
        ScVal::Bytes(ScBytes(vec![seed; 32].try_into().unwrap()))
    }

    fn account_address_val(pubkey_bytes: [u8; 32]) -> ScVal {
        ScVal::Address(ScAddress::Account(AccountId(
            PublicKey::PublicKeyTypeEd25519(Uint256(pubkey_bytes)),
        )))
    }

    fn make_raw_event(topics: Vec<ScVal>, value: ScVal) -> RawEvent {
        #[allow(deprecated)]
        RawEvent {
            event_type: "contract".to_string(),
            ledger: 100,
            ledger_closed_at: "2026-07-11T00:00:00Z".to_string(),
            contract_id: "CC4BP273CO73T6AXOMILHWXF37EJ5B766JEOKCYTET4YBHE3FX46GYNI".to_string(),
            id: "0000000000000001-0000000001".to_string(),
            operation_index: None,
            transaction_index: None,
            tx_hash: Some(
                "abc123def456abc123def456abc123def456abc123def456abc123def456abc1".to_string(),
            ),
            is_successful_contract_call: None,
            topic: topics.iter().map(encode_scval).collect(),
            value: encode_scval(&value),
        }
    }

    // -----------------------------------------------------------------------
    // register_project
    // -----------------------------------------------------------------------

    #[test]
    fn decodes_register_project_event() {
        let pubkey = [0x42u8; 32];
        let register_value = ScVal::Vec(Some(ScVec(
            VecM::try_from(vec![account_address_val(pubkey), ScVal::U32(2)]).unwrap(),
        )));
        let raw = make_raw_event(
            vec![symbol_val("register"), bytes32_val(0xAB)],
            register_value,
        );

        let decoded = decode_event(&raw).unwrap().unwrap();

        match decoded {
            DecodedEvent::RegisterProject {
                project_id,
                owner_address,
                receiver_count,
            } => {
                assert_eq!(project_id, "ab".repeat(32));
                assert!(
                    owner_address.starts_with('G'),
                    "should be a G... strkey, got: {owner_address}"
                );
                assert_eq!(receiver_count, 2);
            }
            other => panic!("expected RegisterProject, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // update_splits
    // -----------------------------------------------------------------------

    #[test]
    fn decodes_update_splits_event() {
        let raw = make_raw_event(vec![symbol_val("update"), bytes32_val(0x01)], ScVal::U32(3));

        let decoded = decode_event(&raw).unwrap().unwrap();

        match decoded {
            DecodedEvent::UpdateSplits {
                project_id,
                new_receiver_count,
            } => {
                assert_eq!(project_id, "01".repeat(32));
                assert_eq!(new_receiver_count, 3);
            }
            other => panic!("expected UpdateSplits, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Error cases
    // -----------------------------------------------------------------------

    #[test]
    fn rejects_too_few_topics() {
        let raw = make_raw_event(vec![symbol_val("register")], ScVal::U32(0));
        let err = decode_event(&raw).unwrap_err();
        assert!(
            matches!(err, DecodeError::TooFewTopics { .. }),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_non_symbol_first_topic() {
        let raw = make_raw_event(vec![ScVal::U32(99), bytes32_val(0x01)], ScVal::U32(0));
        let err = decode_event(&raw).unwrap_err();
        assert!(
            matches!(err, DecodeError::NotASymbol(_)),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_unknown_event_type() {
        let raw = make_raw_event(
            vec![symbol_val("frobnicate"), bytes32_val(0x01)],
            ScVal::U32(0),
        );
        let err = decode_event(&raw).unwrap_err();
        assert!(
            matches!(err, DecodeError::UnknownEventType(ref s) if s == "frobnicate"),
            "unexpected error: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // deposit
    // -----------------------------------------------------------------------

    #[test]
    fn decodes_deposit_event() {
        use stellar_xdr::{Hash, Int128Parts, ScAddress};

        let token_pubkey = [0xCC_u8; 32];
        // Deposit: topics = [symbol("deposit"), project_id_bytes, token_address], value = i128
        let amount: i128 = 1_000_000; // 1 USDC in micro-units
        // Encode the i128 as Int128Parts
        let hi = (amount >> 64) as i64;
        let lo = amount as u64;

        // Use a contract address for the token
        let token_addr = ScVal::Address(ScAddress::Contract(stellar_xdr::ContractId(Hash(
            token_pubkey,
        ))));

        let mut raw = make_raw_event(
            vec![symbol_val("deposit"), bytes32_val(0xAA)],
            ScVal::I128(Int128Parts { hi, lo }),
        );
        // Inject the third topic (token address) — make_raw_event only adds 2 topics
        raw.topic.push(encode_scval(&token_addr));

        let decoded = decode_event(&raw).unwrap().unwrap();

        match decoded {
            DecodedEvent::Deposit {
                project_id,
                token_address,
                amount: decoded_amount,
            } => {
                assert_eq!(project_id, "aa".repeat(32));
                assert!(
                    token_address.starts_with('C'),
                    "token should be a C... strkey, got: {token_address}"
                );
                assert_eq!(decoded_amount, amount);
            }
            other => panic!("expected Deposit, got {other:?}"),
        }
    }

    #[test]
    fn deposit_rejects_missing_token_topic() {
        use stellar_xdr::Int128Parts;
        // Only 2 topics — deposit requires 3
        let raw = make_raw_event(
            vec![symbol_val("deposit"), bytes32_val(0x01)],
            ScVal::I128(Int128Parts { hi: 0, lo: 100 }),
        );
        let err = decode_event(&raw).unwrap_err();
        assert!(
            matches!(err, DecodeError::TooFewTopics { .. }),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn deposit_i128_roundtrip() {
        // Verify hi/lo reconstruction for a large negative value (stress test sign extension).
        use stellar_xdr::{Hash, Int128Parts, ScAddress};

        let amount: i128 = -42_000_000_000_000_i128;
        let hi = (amount >> 64) as i64;
        let lo = amount as u64;

        let token_addr = ScVal::Address(ScAddress::Contract(stellar_xdr::ContractId(Hash(
            [0xAB; 32],
        ))));

        let mut raw = make_raw_event(
            vec![symbol_val("deposit"), bytes32_val(0x01)],
            ScVal::I128(Int128Parts { hi, lo }),
        );
        raw.topic.push(encode_scval(&token_addr));

        let decoded = decode_event(&raw).unwrap().unwrap();
        match decoded {
            DecodedEvent::Deposit {
                amount: decoded_amount,
                ..
            } => {
                assert_eq!(
                    decoded_amount, amount,
                    "i128 hi/lo reconstruction must be exact"
                );
            }
            other => panic!("expected Deposit, got {other:?}"),
        }
    }
}
