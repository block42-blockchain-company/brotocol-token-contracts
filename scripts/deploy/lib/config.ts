import * as fs from 'fs';

export function loadConfig(chainID: string): Config {
    const data = fs.readFileSync(`./config/${chainID}.json`, 'utf8');
    return JSON.parse(data);
}

export interface Config {
    bro_token: BroTokenConfig,
    airdrop: AirdropConfig,
    vesting: VestingConfig,
    bbro_minter: BbroMinterConfig,
    bbro_token: BbroTokenConfig,
    rewards: RewardsPoolConfig,
    treasury: TreasuryConfig,
    epoch_manager: EpochManagerConfig,
    stakingv1: StakingV1Config,
    bondingv1: BondingV1Config,
    distributorv1: DistributorV1Config,
    bro_distributions: BroDistributionAmountsConfig,
};

export interface BroTokenConfig {
    name: string,
    symbol: string,
    decimals: number,
    initial_balances: [{
        address: string,
        amount: string,
    }],
}

export interface AirdropConfig {
    owner: string,
    bro_token: string,
}

export interface VestingConfig {
    owner: string,
    bro_token: string,
    genesis_time: number,
}

export interface BbroMinterConfig {
    owner: string,
    whitelist: string[]
}

export interface BbroTokenConfig {
    name: string,
    symbol: string,
    decimals: number,
    initial_balances: [],
    mint: {
        minter: string,
    },
}

export interface RewardsPoolConfig {
    owner: string,
    bro_token: string,
    spend_limit: string,
    whitelist: string[],
}

export interface TreasuryConfig {
    owner: string,
}

export interface EpochManagerConfig {
    owner: string,
    epoch: number,
    blocks_per_year: number,
    bbro_emission_rate: string,
}

export interface StakingV1Config {
    bro_token: string,
    rewards_pool_contract: string,
    bbro_minter_contract: string,
    epoch_manager_contract: string,
    unbond_period_blocks: number,
}

// TODO: create mock contract for astro pool and lp token queries
export interface BondingV1Config {
    owner: string,
    bro_token: string,
    lp_token: string,
    treasury_contract: string,
    astroport_factory: string,
    ust_bonding_reward_ratio: string,
    ust_bonding_discount: string,
    lp_bonding_discount: string,
    vesting_period_blocks: number,
}

export interface DistributorV1Config {
    owner: string,
    distribution_genesis_block: number,
    epoch_manager_contract: string,
    rewards_contract: string,
    staking_contract: string,
    staking_distribution_amount: string,
    bonding_contract: string,
    bonding_distribution_amount: string,
}

export interface BroDistributionAmountsConfig {
    airdrop: string,
    vesting: string,
    rewards: string,
}