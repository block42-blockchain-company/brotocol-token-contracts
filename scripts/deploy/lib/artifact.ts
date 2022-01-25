import * as fs from 'fs';
import path from 'path'

const ARTIFACTS_PATH = './artifacts'

export interface Artifact {
    network: string,
    bro_token: string,
    airdrop: string,
    vesting: string,
    bbro_token: string,
    bbro_minter: string,
    rewards_pool: string,
    mvp_treasury: string,
    epoch_manager: string,
    staking_v1: string,
    bonding_v1: string,
    distributor_v1: string,
}

export function loadArtifact(name: string = "artifact"): Artifact {
    try {
        const data = fs.readFileSync(path.join(ARTIFACTS_PATH, `${name}.json`), 'utf8');
        return JSON.parse(data);
    } catch (e) {
        return <Artifact>{};
    }
}

export function writeArtifact(data: Artifact, name: string = "artifact") {
    fs.writeFileSync(path.join(ARTIFACTS_PATH, `${name}.json`), JSON.stringify(data, null, 2))
}