use soroban_sdk::{Address, Env, Vec, String};
use crate::types::{Error, BuybackCampaign, BuybackStep, CampaignStatus, StepStatus};
use crate::storage;
use crate::events;

pub fn create_buyback_campaign(
    env: Env,
    creator: Address,
    token_address: Address,
    total_amount: i128,
    steps: Vec<i128>,
) -> Result<u64, Error> {
    creator.require_auth();

    if total_amount <= 0 {
        return Err(Error::InvalidParameters);
    }

    let campaign_id = storage::get_next_campaign_id(&env);
    
    let campaign = BuybackCampaign {
        id: campaign_id,
        creator: creator.clone(),
        token_address: token_address.clone(),
        total_amount,
        executed_amount: 0,
        current_step: 0,
        total_steps: steps.len(),
        status: CampaignStatus::Active,
        created_at: env.ledger().timestamp(),
    };

    storage::set_buyback_campaign(&env, campaign_id, &campaign);
    
    for (idx, amount) in steps.iter().enumerate() {
        let step = BuybackStep {
            step_number: idx as u32,
            amount,
            status: StepStatus::Pending,
            executed_at: None,
            tx_hash: None,
        };
        storage::set_campaign_step(&env, campaign_id, idx as u32, &step);
    }

    events::emit_campaign_created(&env, campaign_id, &creator, &token_address, total_amount);

    Ok(campaign_id)
}

pub fn execute_buyback_step(
    env: Env,
    executor: Address,
    campaign_id: u64,
) -> Result<(), Error> {
    executor.require_auth();

    let mut campaign = storage::get_buyback_campaign(&env, campaign_id)
        .ok_or(Error::TokenNotFound)?; // Reusing TokenNotFound for CampaignNotFound

    if campaign.status != CampaignStatus::Active {
        return Err(Error::ContractPaused); // Reusing ContractPaused for CampaignNotActive
    }

    if campaign.creator != executor {
        return Err(Error::Unauthorized);
    }

    if campaign.current_step >= campaign.total_steps {
        return Err(Error::InvalidParameters); // Reusing InvalidParameters for AllStepsCompleted
    }

    let mut step = storage::get_campaign_step(&env, campaign_id, campaign.current_step)
        .ok_or(Error::TokenNotFound)?; // Reusing TokenNotFound for StepNotFound

    if step.status != StepStatus::Pending {
        return Err(Error::ChangeAlreadyExecuted); // Reusing ChangeAlreadyExecuted for StepAlreadyExecuted
    }

    // Execute the buyback (burn tokens)
    let token_client = soroban_sdk::token::Client::new(&env, &campaign.token_address);
    token_client.burn(&executor, &step.amount);

    // Update step status
    step.status = StepStatus::Completed;
    step.executed_at = Some(env.ledger().timestamp());
    storage::set_campaign_step(&env, campaign_id, campaign.current_step, &step);

    // Update campaign
    campaign.executed_amount += step.amount;
    campaign.current_step += 1;

    if campaign.current_step >= campaign.total_steps {
        campaign.status = CampaignStatus::Completed;
    }

    storage::set_buyback_campaign(&env, campaign_id, &campaign);

    events::emit_step_executed(
        &env,
        campaign_id,
        campaign.current_step - 1,
        step.amount,
        &executor,
    );

    Ok(())
    }

pub fn get_campaign(env: Env, campaign_id: u64) -> Result<BuybackCampaign, Error> {
    storage::get_buyback_campaign(&env, campaign_id).ok_or(Error::TokenNotFound)
    }

pub fn get_campaign_step(
    env: Env,
    campaign_id: u64,
    step_number: u32,
    ) -> Result<BuybackStep, Error> {
    storage::get_campaign_step(&env, campaign_id, step_number).ok_or(Error::TokenNotFound)
    }

pub fn cancel_campaign(env: Env, creator: Address, campaign_id: u64) -> Result<(), Error> {
    creator.require_auth();

    let mut campaign = storage::get_buyback_campaign(&env, campaign_id)
        .ok_or(Error::TokenNotFound)?;

    if campaign.creator != creator {
        return Err(Error::Unauthorized);
    }

    if campaign.status != CampaignStatus::Active {
        return Err(Error::ContractPaused);
    }

    campaign.status = CampaignStatus::Cancelled;
    storage::set_buyback_campaign(&env, campaign_id, &campaign);

    events::emit_campaign_cancelled(&env, campaign_id, &creator);

    Ok(())
    }

