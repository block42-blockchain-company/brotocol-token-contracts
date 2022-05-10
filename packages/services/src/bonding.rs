use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::{Cw20ReceiveMsg, Expiration};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// bro token address
    pub bro_token: String,
    /// rewards pool address
    pub rewards_pool_contract: String,
    /// treasury contract address
    pub treasury_contract: String,
    /// astroport factory contract address
    pub astroport_factory: String,
    /// price oracle contract address
    pub oracle_contract: String,
    /// discount percentage for ust bonding
    pub ust_bonding_discount: Decimal,
    /// minimum amount of bro to receive via bonding
    pub min_bro_payout: Uint128,
    /// bonding mode
    pub bonding_mode: BondingModeMsg,
}

/// ## BondingModeMsg
/// This structure describes the bonding contract mode.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BondingModeMsg {
    /// ## Description
    /// Enables both ust and lp bonding option.
    /// Exchanged bro tokens will become claimable after vesting period.
    Normal {
        /// distributed reward percentage for ust bonding balance
        ust_bonding_reward_ratio: Decimal,
        /// bro/ust lp token address
        lp_token: String,
        /// discount percentage for lp bonding
        lp_bonding_discount: Decimal,
        /// vesting period for withdrawal
        vesting_period_blocks: u64,
    },
    /// ## Description
    /// Enables only ust bonding option.
    /// Exchanged bro tokens will be locked in staking contract for configured amount of epochs
    /// and then claimable with extra bro/bbro reward from it.
    Community {
        /// staking contract address
        staking_contract: String,
        /// how many epochs specified amount will be locked
        epochs_locked: u64,
    },
}

/// ## ExecuteMsg
/// This structure describes the execute messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ## Description
    /// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received
    /// template.
    Receive(Cw20ReceiveMsg),
    /// ## Description
    /// Bond bro tokens by providing ust amount.
    UstBond {},
    /// ## Description
    /// Claim available reward amount.
    Claim {},
    /// ## Description
    /// Updates contract settings
    /// ## Executor
    /// Only owner can execute this function
    UpdateConfig {
        /// rewards pool address
        rewards_pool_contract: Option<String>,
        /// new treasury contract address
        treasury_contract: Option<String>,
        /// new astroport factory address
        astroport_factory: Option<String>,
        /// new price oracle contract address
        oracle_contract: Option<String>,
        /// new discount percentage for ust bonding
        ust_bonding_discount: Option<Decimal>,
        /// new minimum amount of bro to receive via bonding
        min_bro_payout: Option<Uint128>,
    },
    /// ## Description
    /// Updates specific settings for bonding mode config
    /// ## Executor
    /// Only owner can execute this function
    UpdateBondingModeConfig {
        /// normal bonding mode: new distributed reward percentage for ust bonding balance
        ust_bonding_reward_ratio_normal: Option<Decimal>,
        /// normal bonding mode: new bro/ust lp token address
        lp_token_normal: Option<String>,
        /// normal bonding mode: new discount percentage for lp bonding
        lp_bonding_discount_normal: Option<Decimal>,
        /// normal bonding mode: new vesting period for withdrawal
        vesting_period_blocks_normal: Option<u64>,
        /// community bonding mode: new staking contract address
        staking_contract_community: Option<String>,
        /// community bonding mode: new amount of epochs for lockup
        epochs_locked_community: Option<u64>,
    },
    /// ## Description
    /// Creates an offer for a new owner.
    /// The validity period of the offer is set in the `expires_in_blocks` variable
    /// ## Executor
    /// Only owner can execute this function
    ProposeNewOwner {
        /// new contract owner
        new_owner: String,
        /// expiration period in blocks
        expires_in_blocks: u64,
    },
    /// ## Description
    /// Removes the existing offer for the new owner
    /// ## Executor
    /// Only owner can execute this function
    DropOwnershipProposal {},
    /// ## Description
    /// Used to claim(approve) new owner proposal, thus changing contract's owner
    /// ## Executor
    /// Only address proposed as a new owner can execute this function
    ClaimOwnership {},
}

/// ## Cw20HookMsg
/// This structure describes the cw20 receive hook messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// ## Description
    /// Distributes received reward
    DistributeReward {},
    /// ## Description
    /// Bond bro tokens by providing lp token amount.
    LpBond {},
}

/// ## QueryMsg
/// This structure describes the query messages of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Returns staking contract config in the [`ConfigResponse`] object
    Config {},
    /// ## Description
    /// Returns staking contract state in the [`StateResponse`] object
    State {},
    /// ## Description
    /// Returns available claims for bonder by specified address
    /// in the [`ClaimsResponse`] object
    Claims {
        /// bonder address
        address: String,
    },
    /// ## Description
    /// Returns simulated bro bond using specified uusd amount in the [`SimulateExchangeResponse`] object
    SimulateUstBond { uusd_amount: Uint128 },
    /// ## Description
    /// Returns simulated bro bond using specified ust/bro lp token amount in the [`SimulateExchangeResponse`] object
    SimulateLpBond { lp_amount: Uint128 },
    /// ## Description
    /// Returns information about created ownership proposal in the [`OwnershipProposalResponse`] object
    /// otherwise returns not-found error
    OwnershipProposal {},
}

/// ## MigrateMsg
/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    /// params for performing migration
    pub params: Binary,
}

/// ## ConfigResponse
/// This structure describes the fields for config response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// bro token address
    pub bro_token: String,
    /// rewards pool address
    pub rewards_pool_contract: String,
    /// treasury contract address
    pub treasury_contract: String,
    /// astroport factory contract address
    pub astroport_factory: String,
    /// price oracle contract address
    pub oracle_contract: String,
    /// discount percentage for ust bonding
    pub ust_bonding_discount: Decimal,
    /// minimum amount of bro to receive via bonding
    pub min_bro_payout: Uint128,
    /// bonding mode
    pub bonding_mode: BondingModeMsg,
}

/// ## StateResponse
/// This structure describes the fields for state response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    /// available bro balance for ust bonding
    pub ust_bonding_balance: Uint128,
    /// available bro balance for lp token bonding
    pub lp_bonding_balance: Uint128,
    /// bonded amount of bro tokens
    pub bonded_bro_amount: Uint128,
}

/// ## ClaimInfoResponse
/// This structure describes the fields for claim info response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimInfoResponse {
    /// bond type
    pub bond_type: String,
    /// amount to claim
    pub amount: Uint128,
    /// block at which amount can be claimed
    pub claimable_at: Expiration,
}

/// ## ClaimsResponse
/// This structure describes the fields for claims response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimsResponse {
    /// a list of bonder claims
    pub claims: Vec<ClaimInfoResponse>,
}

/// ## SimulateExchangeResponse
/// This structure describes the fields for simulated exchange response.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulateExchangeResponse {
    pub bro_payout: Uint128,
    pub can_be_exchanged: bool,
}
