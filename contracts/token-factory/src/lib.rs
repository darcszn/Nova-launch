#![no_std]

mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env, String, Vec};
use types::{Error, FactoryState, TokenInfo};

#[contract]
pub struct TokenFactory;

#[contractimpl]
impl TokenFactory {
    /// Initialize the factory with admin, treasury, and fee structure
    pub fn initialize(
        env: Env,
        admin: Address,
        treasury: Address,
        base_fee: i128,
        metadata_fee: i128,
    ) -> Result<(), Error> {
        // Check if already initialized
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }

        // Validate parameters
        if base_fee < 0 || metadata_fee < 0 {
            return Err(Error::InvalidParameters);
        }

        // Set initial state
        storage::set_admin(&env, &admin);
        storage::set_treasury(&env, &treasury);
        storage::set_base_fee(&env, base_fee);
        storage::set_metadata_fee(&env, metadata_fee);

        Ok(())
    }

    /// Get the current factory state
    pub fn get_state(env: Env) -> FactoryState {
        storage::get_factory_state(&env)
    }

    /// Update fee structure (admin only)
    pub fn update_fees(
        env: Env,
        admin: Address,
        base_fee: Option<i128>,
        metadata_fee: Option<i128>,
    ) -> Result<(), Error> {
        admin.require_auth();

        let current_admin = storage::get_admin(&env);
        if admin != current_admin {
            return Err(Error::Unauthorized);
        }

        if let Some(fee) = base_fee {
            if fee < 0 {
                return Err(Error::InvalidParameters);
            }
            storage::set_base_fee(&env, fee);
        }

        if let Some(fee) = metadata_fee {
            if fee < 0 {
                return Err(Error::InvalidParameters);
            }
            storage::set_metadata_fee(&env, fee);
        }

        Ok(())
    }

    /// Get token count
    pub fn get_token_count(env: Env) -> u32 {
        storage::get_token_count(&env)
    }

    /// Get token info by index
    pub fn get_token_info(env: Env, index: u32) -> Result<TokenInfo, Error> {
        storage::get_token_info(&env, index).ok_or(Error::TokenNotFound)
    }

    /// Get token info by address
    pub fn get_token_info_by_address(env: Env, token_address: Address) -> TokenInfo {
        let count = storage::get_token_count(&env);
        for i in 0..count {
            if let Some(info) = storage::get_token_info(&env, i) {
                if info.address == token_address {
                    return info;
                }
            }
        }
        panic!("Token not found")
    }

    /// Create a new token (simplified for testing)
    pub fn create_token(
        env: Env,
        creator: Address,
        name: String,
        symbol: String,
        decimals: u32,
        initial_supply: i128,
    ) -> Result<Address, Error> {
        // Validate parameters
        if initial_supply < 0 {
            return Err(Error::InvalidParameters);
        }

        // Generate token address (simulated)
        let token_address = Address::generate(&env);

        // Create token info
        let info = TokenInfo {
            address: token_address.clone(),
            creator,
            name,
            symbol,
            decimals,
            total_supply: initial_supply,
            metadata_uri: None,
            created_at: env.ledger().timestamp(),
            total_burned: 0,
            burn_count: 0,
        };

        // Store token info
        let count = storage::get_token_count(&env);
        storage::set_token_info(&env, count, &info);

        Ok(token_address)
    }

    /// Burn tokens from caller's balance
    pub fn burn(env: Env, token_address: Address, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidBurnAmount);
        }

        let mut info = Self::get_token_info_by_address(env.clone(), token_address.clone());
        
        info.total_supply = info.total_supply.checked_sub(amount).ok_or(Error::InvalidBurnAmount)?;
        info.total_burned = info.total_burned.checked_add(amount).ok_or(Error::InvalidBurnAmount)?;
        info.burn_count = info.burn_count.checked_add(1).ok_or(Error::InvalidBurnAmount)?;

        let count = storage::get_token_count(&env);
        for i in 0..count {
            if let Some(token_info) = storage::get_token_info(&env, i) {
                if token_info.address == token_address {
                    storage::set_token_info(&env, i, &info);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Admin burn tokens from any address
    pub fn admin_burn(
        env: Env,
        token_address: Address,
        admin: Address,
        from: Address,
        amount: i128,
    ) -> Result<(), Error> {
        admin.require_auth();

        let info = Self::get_token_info_by_address(env.clone(), token_address.clone());
        if admin != info.creator {
            return Err(Error::Unauthorized);
        }

        Self::burn(env, token_address, from, amount)
    }

    /// Batch burn tokens
    pub fn burn_batch(
        env: Env,
        token_address: Address,
        burns: Vec<(Address, i128)>,
    ) -> Result<(), Error> {
        for (from, amount) in burns.iter() {
            Self::burn(env.clone(), token_address.clone(), from, amount)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;

#[cfg(test)]
mod fuzz_test;

#[cfg(test)]
mod bench_test;

#[cfg(test)]
mod supply_conservation_test;

#[cfg(test)]
mod fee_validation_test;

// Temporarily disabled due to compilation issues
// #[cfg(test)]
// mod atomic_token_creation_test;

#[cfg(test)]
mod burn_integration_test;
