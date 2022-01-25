import * as fs from 'fs';

export function loadConfig(chainID: string): Config {
    const data = fs.readFileSync(`./config/${chainID}.json`, 'utf8');
    return JSON.parse(data);
}

export interface Config {
    bro_token: BroTokenConfig,
    airdrop: AirdropConfig,
    vesting: VestingConfig,
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
    owner: String,
    bro_token: String,
}

export interface VestingConfig {
    owner: String,
    bro_token: String,
    genesis_time: number,
}
