#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env};

/// Lifecycle of a single harvest escrow.
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum EscrowStatus {
    Pending,
    Delivered,
    Cancelled,
}

/// One farmer-buyer trade held in escrow.
#[contracttype]
#[derive(Clone)]
pub struct Escrow {
    pub farmer: Address,
    pub buyer: Address,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowStatus,
}

#[contracttype]
pub enum DataKey {
    Escrow(u64),
    NextId,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Error {
    NotFound = 1,
    NotPending = 2,
    Unauthorized = 3,
}

#[contract]
pub struct HarvestPayContract;

#[contractimpl]
impl HarvestPayContract {
    /// Buyer (trader) opens and funds an escrow ahead of a harvest delivery.
    /// On-chain: pulls `amount` of `token` from the buyer into the contract
    /// and records the escrow so it can be tracked by id.
    pub fn create_escrow(env: Env, buyer: Address, farmer: Address, token: Address, amount: i128) -> u64 {
        buyer.require_auth();

        let id: u64 = env.storage().instance().get(&DataKey::NextId).unwrap_or(0);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&buyer, &env.current_contract_address(), &amount);

        let escrow = Escrow {
            farmer: farmer.clone(),
            buyer: buyer.clone(),
            token: token.clone(),
            amount,
            status: EscrowStatus::Pending,
        };

        env.storage().persistent().set(&DataKey::Escrow(id), &escrow);
        env.storage().instance().set(&DataKey::NextId, &(id + 1));

        env.events().publish((symbol_short!("created"), id), amount);
        id
    }

    /// Buyer confirms physical delivery (e.g. scans farmer's QR at the trading post).
    /// On-chain: releases the escrowed funds straight to the farmer and closes the escrow.
    pub fn confirm_delivery(env: Env, escrow_id: u64, buyer: Address) -> Result<(), Error> {
        buyer.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .ok_or(Error::NotFound)?;

        if escrow.buyer != buyer {
            return Err(Error::Unauthorized);
        }
        if escrow.status != EscrowStatus::Pending {
            return Err(Error::NotPending);
        }

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(&env.current_contract_address(), &escrow.farmer, &escrow.amount);

        escrow.status = EscrowStatus::Delivered;
        env.storage().persistent().set(&DataKey::Escrow(escrow_id), &escrow);

        env.events().publish((symbol_short!("released"), escrow_id), escrow.amount);
        Ok(())
    }

    /// Buyer cancels an undelivered escrow (e.g. harvest rejected or no-show) and reclaims funds.
    pub fn cancel_escrow(env: Env, escrow_id: u64, buyer: Address) -> Result<(), Error> {
        buyer.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .ok_or(Error::NotFound)?;

        if escrow.buyer != buyer {
            return Err(Error::Unauthorized);
        }
        if escrow.status != EscrowStatus::Pending {
            return Err(Error::NotPending);
        }

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(&env.current_contract_address(), &escrow.buyer, &escrow.amount);

        escrow.status = EscrowStatus::Cancelled;
        env.storage().persistent().set(&DataKey::Escrow(escrow_id), &escrow);

        env.events().publish((symbol_short!("cancelled"), escrow_id), escrow.amount);
        Ok(())
    }

    /// Read-only lookup used by both the farmer and buyer front-ends to display status.
    pub fn get_escrow(env: Env, escrow_id: u64) -> Result<Escrow, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .ok_or(Error::NotFound)
    }
}

#[cfg(test)]
mod test;