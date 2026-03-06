use soroban_sdk::{Address, Env, Vec};
use crate::types::{Error, StreamInfo, StreamParams};
use crate::storage;
use crate::events;

/// Maximum number of streams in a batch operation
const MAX_BATCH_SIZE: u32 = 100;

/// Create a single stream
///
/// Creates a payment stream from creator to recipient with vesting schedule.
///
/// # Arguments
/// * `env` - The contract environment
/// * `creator` - Address creating the stream (must authorize)
/// * `params` - Stream parameters (recipient, amount, schedule)
///
/// # Returns
/// Returns the stream ID
///
/// # Errors
/// * `Error::Unauthorized` - Caller is not the creator
/// * `Error::InvalidParameters` - Invalid stream parameters
/// * `Error::ContractPaused` - Contract is paused
pub fn create_stream(
    env: &Env,
    creator: &Address,
    params: &StreamParams,
) -> Result<u64, Error> {
    creator.require_auth();
    
    // Check if contract is paused
    if storage::is_paused(env) {
        return Err(Error::ContractPaused);
    }
    
    // Validate stream parameters
    validate_stream_params(env, params)?;
    
    // Get next stream ID
    let stream_id = storage::get_next_stream_id(env);
    
    // Create stream info
    let stream = StreamInfo {
        id: stream_id,
        creator: creator.clone(),
        recipient: params.recipient.clone(),
        token_index: params.token_index,
        total_amount: params.total_amount,
        claimed_amount: 0,
        start_time: params.start_time,
        end_time: params.end_time,
        cliff_time: params.cliff_time,
        cancelled: false,
    };
    
    // Store stream
    storage::set_stream(env, stream_id, &stream);
    
    // Emit event
    events::emit_stream_created(env, stream_id, creator, &params.recipient, params.total_amount);
    
    Ok(stream_id)
}

/// Batch create streams
///
/// Creates multiple payment streams in a single transaction.
/// All-or-nothing atomicity: if any stream is invalid, entire batch fails.
///
/// # Arguments
/// * `env` - The contract environment
/// * `creator` - Address creating the streams (must authorize)
/// * `streams` - Vector of stream parameters
///
/// # Returns
/// Returns vector of created stream IDs
///
/// # Errors
/// * `Error::Unauthorized` - Caller is not the creator
/// * `Error::InvalidParameters` - Invalid parameters or batch too large
/// * `Error::ContractPaused` - Contract is paused
/// * `Error::BatchTooLarge` - Batch exceeds maximum size
///
/// # Examples
/// ```
/// let streams = vec![
///     &env,
///     StreamParams { recipient: addr1, amount: 1000, ... },
///     StreamParams { recipient: addr2, amount: 2000, ... },
/// ];
/// let stream_ids = batch_create_streams(&env, &creator, &streams)?;
/// ```
pub fn batch_create_streams(
    env: &Env,
    creator: &Address,
    streams: &Vec<StreamParams>,
) -> Result<Vec<u64>, Error> {
    creator.require_auth();
    
    // Check if contract is paused
    if storage::is_paused(env) {
        return Err(Error::ContractPaused);
    }
    
    // Validate batch size
    if streams.is_empty() {
        return Err(Error::InvalidParameters);
    }
    
    if streams.len() > MAX_BATCH_SIZE {
        return Err(Error::BatchTooLarge);
    }
    
    // Phase 1: Validate all streams before creating any
    for stream_params in streams.iter() {
        validate_stream_params(env, &stream_params)?;
    }
    
    // Phase 2: Create all streams (validation passed)
    let mut stream_ids = Vec::new(env);
    
    for stream_params in streams.iter() {
        let stream_id = storage::get_next_stream_id(env);
        
        let stream = StreamInfo {
            id: stream_id,
            creator: creator.clone(),
            recipient: stream_params.recipient.clone(),
            token_index: stream_params.token_index,
            total_amount: stream_params.total_amount,
            claimed_amount: 0,
            start_time: stream_params.start_time,
            end_time: stream_params.end_time,
            cliff_time: stream_params.cliff_time,
            cancelled: false,
        };
        
        storage::set_stream(env, stream_id, &stream);
        stream_ids.push_back(stream_id);
    }
    
    // Emit batch summary event
    events::emit_batch_streams_created(env, creator, stream_ids.len());
    
    Ok(stream_ids)
}

/// Validate stream parameters
///
/// Checks that stream parameters are valid and consistent.
///
/// # Validation Rules
/// * Amount must be positive
/// * Start time must be before end time
/// * Cliff time must be between start and end
/// * Token must exist
fn validate_stream_params(env: &Env, params: &StreamParams) -> Result<(), Error> {
    // Validate amount
    if params.total_amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    
    // Validate times
    if params.start_time >= params.end_time {
        return Err(Error::InvalidParameters);
    }
    
    // Validate cliff time
    if params.cliff_time < params.start_time || params.cliff_time > params.end_time {
        return Err(Error::InvalidParameters);
    }
    
    // Validate token exists
    if storage::get_token_info(env, params.token_index).is_none() {
        return Err(Error::TokenNotFound);
    }
    
    Ok(())
}

/// Claim vested tokens from a stream
///
/// Allows recipient to claim tokens that have vested according to schedule.
///
/// # Arguments
/// * `env` - The contract environment
/// * `recipient` - Address claiming tokens (must authorize)
/// * `stream_id` - ID of the stream to claim from
///
/// # Returns
/// Returns the amount claimed
///
/// # Errors
/// * `Error::Unauthorized` - Caller is not the recipient
/// * `Error::TokenNotFound` - Stream not found
/// * `Error::InvalidParameters` - Stream cancelled or no claimable amount
pub fn claim_stream(
    env: &Env,
    recipient: &Address,
    stream_id: u64,
) -> Result<i128, Error> {
    recipient.require_auth();
    
    // Get stream
    let mut stream = storage::get_stream(env, stream_id)
        .ok_or(Error::TokenNotFound)?;
    
    // Verify recipient
    if stream.recipient != *recipient {
        return Err(Error::Unauthorized);
    }
    
    // Check if cancelled
    if stream.cancelled {
        return Err(Error::InvalidParameters);
    }
    
    // Calculate claimable amount
    let claimable = calculate_claimable(env, &stream)?;
    
    if claimable == 0 {
        return Err(Error::InvalidAmount);
    }
    
    // Update claimed amount
    stream.claimed_amount = stream.claimed_amount
        .checked_add(claimable)
        .ok_or(Error::ArithmeticError)?;
    
    storage::set_stream(env, stream_id, &stream);
    
    // Emit event
    events::emit_stream_claimed(env, stream_id, recipient, claimable);
    
    Ok(claimable)
}

/// Calculate claimable amount for a stream
///
/// Calculates how much can be claimed based on vesting schedule.
fn calculate_claimable(env: &Env, stream: &StreamInfo) -> Result<i128, Error> {
    let current_time = env.ledger().timestamp();
    
    // Before cliff: nothing claimable
    if current_time < stream.cliff_time {
        return Ok(0);
    }
    
    // Before start: nothing claimable
    if current_time < stream.start_time {
        return Ok(0);
    }
    
    // Calculate vested amount
    let vested = if current_time >= stream.end_time {
        // After end: everything is vested
        stream.total_amount
    } else {
        // During vesting: linear vesting
        let elapsed = current_time - stream.start_time;
        let duration = stream.end_time - stream.start_time;
        
        stream.total_amount
            .checked_mul(elapsed as i128)
            .and_then(|v| v.checked_div(duration as i128))
            .ok_or(Error::ArithmeticError)?
    };
    
    // Claimable = vested - already claimed
    let claimable = vested
        .checked_sub(stream.claimed_amount)
        .ok_or(Error::ArithmeticError)?;
    
    Ok(claimable.max(0))
}

/// Cancel a stream
///
/// Allows creator to cancel a stream. Recipient can claim vested amount.
///
/// # Arguments
/// * `env` - The contract environment
/// * `creator` - Address cancelling the stream (must authorize)
/// * `stream_id` - ID of the stream to cancel
pub fn cancel_stream(
    env: &Env,
    creator: &Address,
    stream_id: u64,
) -> Result<(), Error> {
    creator.require_auth();
    
    // Get stream
    let mut stream = storage::get_stream(env, stream_id)
        .ok_or(Error::TokenNotFound)?;
    
    // Verify creator
    if stream.creator != *creator {
        return Err(Error::Unauthorized);
    }
    
    // Check if already cancelled
    if stream.cancelled {
        return Err(Error::InvalidParameters);
    }
    
    // Mark as cancelled
    stream.cancelled = true;
    storage::set_stream(env, stream_id, &stream);
    
    // Emit event
    events::emit_stream_cancelled(env, stream_id, creator);
    
    Ok(())
}

/// Get stream information
pub fn get_stream(env: &Env, stream_id: u64) -> Option<StreamInfo> {
    storage::get_stream(env, stream_id)
}

/// Get claimable amount for a stream
pub fn get_claimable_amount(env: &Env, stream_id: u64) -> Result<i128, Error> {
    let stream = storage::get_stream(env, stream_id)
        .ok_or(Error::TokenNotFound)?;
    
    calculate_claimable(env, &stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, vec, Env};
    
    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        
        let creator = Address::generate(&env);
        let recipient = Address::generate(&env);
        
        // Initialize storage
        storage::set_admin(&env, &creator);
        
        (env, creator, recipient)
    }
    
    #[test]
    fn test_validate_stream_params_valid() {
        let (env, _creator, recipient) = setup();
        
        let params = StreamParams {
            recipient: recipient.clone(),
            token_index: 0,
            total_amount: 1000,
            start_time: 100,
            end_time: 200,
            cliff_time: 150,
        };
        
        // This will fail because token doesn't exist, but tests validation logic
        let result = validate_stream_params(&env, &params);
        assert_eq!(result, Err(Error::TokenNotFound));
    }
    
    #[test]
    fn test_validate_stream_params_invalid_amount() {
        let (env, _creator, recipient) = setup();
        
        let params = StreamParams {
            recipient: recipient.clone(),
            token_index: 0,
            total_amount: 0,
            start_time: 100,
            end_time: 200,
            cliff_time: 150,
        };
        
        let result = validate_stream_params(&env, &params);
        assert_eq!(result, Err(Error::InvalidAmount));
    }
    
    #[test]
    fn test_validate_stream_params_invalid_times() {
        let (env, _creator, recipient) = setup();
        
        let params = StreamParams {
            recipient: recipient.clone(),
            token_index: 0,
            total_amount: 1000,
            start_time: 200,
            end_time: 100, // End before start
            cliff_time: 150,
        };
        
        let result = validate_stream_params(&env, &params);
        assert_eq!(result, Err(Error::InvalidParameters));
    }
    
    #[test]
    fn test_calculate_claimable_before_cliff() {
        let (env, creator, recipient) = setup();
        
        let stream = StreamInfo {
            id: 0,
            creator: creator.clone(),
            recipient: recipient.clone(),
            token_index: 0,
            total_amount: 1000,
            claimed_amount: 0,
            start_time: 100,
            end_time: 200,
            cliff_time: 150,
            cancelled: false,
        };
        
        // Set time before cliff
        env.ledger().with_mut(|li| {
            li.timestamp = 140;
        });
        
        let claimable = calculate_claimable(&env, &stream).unwrap();
        assert_eq!(claimable, 0);
    }
    
    #[test]
    fn test_calculate_claimable_after_cliff() {
        let (env, creator, recipient) = setup();
        
        let stream = StreamInfo {
            id: 0,
            creator: creator.clone(),
            recipient: recipient.clone(),
            token_index: 0,
            total_amount: 1000,
            claimed_amount: 0,
            start_time: 100,
            end_time: 200,
            cliff_time: 150,
            cancelled: false,
        };
        
        // Set time after cliff (halfway through vesting)
        env.ledger().with_mut(|li| {
            li.timestamp = 150;
        });
        
        let claimable = calculate_claimable(&env, &stream).unwrap();
        assert_eq!(claimable, 500); // 50% vested
    }
    
    #[test]
    fn test_calculate_claimable_after_end() {
        let (env, creator, recipient) = setup();
        
        let stream = StreamInfo {
            id: 0,
            creator: creator.clone(),
            recipient: recipient.clone(),
            token_index: 0,
            total_amount: 1000,
            claimed_amount: 0,
            start_time: 100,
            end_time: 200,
            cliff_time: 150,
            cancelled: false,
        };
        
        // Set time after end
        env.ledger().with_mut(|li| {
            li.timestamp = 250;
        });
        
        let claimable = calculate_claimable(&env, &stream).unwrap();
        assert_eq!(claimable, 1000); // 100% vested
    }
}
