use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::common::OrderBy;

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// bro token address
    pub bro_token: String,
    /// genesis time frame for vesting schedules
    pub genesis_time: u64,
}

/// ## ExecuteMsg
/// This structure describes the execute messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ## Description
    /// Updates contract settings
    /// ## Executor
    /// Only owner can execute this function
    UpdateConfig {
        /// new contract owner
        owner: Option<String>,
        /// new genesis time frame
        genesis_time: Option<u64>,
    },
    /// ## Description
    /// Registers vesting accounts for future distribution
    /// ## Executor
    /// Only owner can execute this function
    RegisterVestingAccounts {
        /// vesting accounts
        vesting_accounts: Vec<VestingAccount>,
    },
    /// ## Description
    /// Claims availalble amount for message sender
    Claim {},
}

/// ## QueryMsg
/// This structure describes the query messages of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Returns vesting contract config in the [`ConfigResponse`] object
    Config {},
    /// ## Description
    /// Returns vesting schedules for specified account in the [`VestingAccountResponse`] object
    VestingAccount { address: String },
    /// ## Description
    /// Returns a list of accounts for given input params in the [`VestingAccountsResponse`] object
    VestingAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    /// ## Description
    /// Returns available amount to claim for specified account in the [`ClaimableAmountResponse`] object
    Claimable { address: String },
}

/// ## MigrateMsg
/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

/// ## VestingAccount
/// This structure describes the basic settings for vesting account.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingAccount {
    /// account address
    pub address: String,
    /// vesting schedules
    pub schedules: Vec<VestingSchedule>,
}

/// ## VestingInfo
/// This structure describes the basic settings for vesting information.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingInfo {
    /// vesting schedules
    pub schedules: Vec<VestingSchedule>,
    /// last rewards claim time
    pub last_claim_time: u64,
}

impl VestingInfo {
    pub fn compute_claim_amount(&self, current_time: u64) -> Uint128 {
        let mut claimable_amount = Uint128::zero();
        for schedule in self.schedules.iter() {
            // claim only passed schedules
            if current_time > schedule.end_time && self.last_claim_time < schedule.end_time {
                claimable_amount += schedule.bro_amount;
            }
        }

        claimable_amount
    }
}

/// ## VestingSchedule
/// This structure describes the basic settings for vesting schedule.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingSchedule {
    /// the start time frame of schedule
    pub start_time: u64,
    /// the end time frame of schedule
    pub end_time: u64,
    /// claimable amount for schedule
    pub bro_amount: Uint128,
}

/// ## ConfigResponse
/// This structure describes the fields for config response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// bro token address
    pub bro_token: String,
    /// genesis time frame for vesting schedules
    pub genesis_time: u64,
}

/// ## VestingAccountResponse
/// This structure describes the fields for vesting account response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingAccountResponse {
    /// vesting account address
    pub address: String,
    /// vesting info for account
    pub info: VestingInfo,
}

/// ## VestingAccountsResponse
/// This structure describes the fields for vesting accounts response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingAccountsResponse {
    /// a list of vesting accounts
    pub vesting_accounts: Vec<VestingAccountResponse>,
}

/// ## ClaimableAmountResponse
/// This structure describes the fields for available amount to claim response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableAmountResponse {
    /// vesting account address
    pub address: String,
    /// available amount to claim
    pub claimable_amount: Uint128,
}
