#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Vec};

#[contracttype]
pub struct Receiver {
    pub address: Address,
    /// Share of incoming funds in basis points (1 bp = 0.01%).
    /// Valid range: 0–10000, where 10000 = 100%.
    pub percentage: u32,
}

#[contracttype]
pub struct Project {
    pub id: BytesN<32>,
    pub owner: Address,
    pub receivers: Vec<Receiver>,
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {}
