import { AirdropConfig, BbroMinterConfig, BbroTokenConfig, BondingV1Config, BroTokenConfig, DistributorV1Config, EpochManagerConfig, RewardsPoolConfig, StakingV1Config, TreasuryConfig, VestingConfig } from "./config.js";
import { TerraClient } from "./client.js";
import { Artifact, writeArtifact } from "./artifact.js";

export interface Contract {
    client: TerraClient;
    artifact: string;
    instantiateMsg: object;
    address: string;

    setArtifactData(artifact: Artifact): void;
};

export async function deployContract(chainID: string, artifact: Artifact, contract: Contract, admin: string) {
    const codeID = await contract.client.storeCode(contract.artifact);
    contract.address = await contract.client.instantiateContract(
        admin,
        codeID,
        contract.instantiateMsg,
        undefined,
    );

    contract.setArtifactData(artifact);
    writeArtifact(artifact, chainID);
}

// bro token
const INITIAL_BRO_BALANCE = 1_000_000_000_000000;
export class BroToken implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: BroTokenConfig;
    public address: string;

    constructor(client: TerraClient, config: BroTokenConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "cw20_base.wasm";
        this.instantiateMsg = this.setInstantiateMsg(config);
        this.address = artifact.bro_token;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.bro_token = this.address;
    }

    public async transfer(address: string, amount: string): Promise<void> {
        await this.client.executeContract(
            this.address,
            {
                transfer: {
                    recipient: address,
                    amount: amount,
                }
            }
        )
    }

    private setInstantiateMsg(config: BroTokenConfig): BroTokenConfig {
        config.initial_balances = [{
            address: this.client.wallet.key.accAddress,
            amount: String(INITIAL_BRO_BALANCE),
        }];
        return config;
    }
}

// airdrop
export class Airdrop implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: AirdropConfig;
    public address: string;
    
    constructor(client: TerraClient, config: AirdropConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_airdrop.wasm";
        this.instantiateMsg = this.setInstantiateMsg(config, artifact);
        this.address = artifact.airdrop;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.airdrop = this.address;
    }

    private setInstantiateMsg(config: AirdropConfig, artifact: Artifact): AirdropConfig {
        config.bro_token = artifact.bro_token;
        return config;
    }
}

// vesting
export class Vesting implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: VestingConfig;
    public address: string;

    constructor(client: TerraClient, config: VestingConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_vesting.wasm";
        this.instantiateMsg = this.setInstantiateMsg(config, artifact);
        this.address = artifact.vesting;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.vesting = this.address;
    }

    private setInstantiateMsg(config: VestingConfig, artifact: Artifact): VestingConfig {
        config.bro_token = artifact.bro_token;
        return config;
    }
}

// bbro-minter
export class BbroMinter implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: BbroMinterConfig;
    public address: string;
    private updateOwner: string; // store owner from initial config to move ownership via update function

    constructor(client: TerraClient, config: BbroMinterConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_bbro_minter.wasm";
        this.updateOwner = config.owner;
        this.instantiateMsg = this.setInstantiateMsg(config);
        this.address = artifact.bbro_minter;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.bbro_minter = this.address;
    }

    public async updateConfig(bbro_token?: string): Promise<void> {
        await this.client.executeContract(
            this.address,
            {
                update_config: {
                    bbro_token: bbro_token,
                }
            }
        );
    }

    public async addMinter(address: string): Promise<void> {
        await this.client.executeContract(
            this.address,
            {
                add_minter: {
                    minter: address,
                }
            }
        )
    }

    public async moveOwnership(): Promise<void> {
        await this.client.executeContract(
            this.address,
            {
                update_config: {
                    owner: this.updateOwner,
                }
            }
        );
    }

    private setInstantiateMsg(config: BbroMinterConfig): BbroMinterConfig {
        config.owner = this.client.wallet.key.accAddress; // we need ownership for setting bbro_token address
        return config;
    }
}

// bbro-token
export class BbroToken implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: BbroTokenConfig;
    public address: string;

    constructor(client: TerraClient, config: BbroTokenConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_bbro_token.wasm";
        this.instantiateMsg = this.setInstantiateMsg(config, artifact);
        this.address = artifact.bbro_token;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.bbro_token = this.address;
    }

    private setInstantiateMsg(config: BbroTokenConfig, artifact: Artifact): BbroTokenConfig {
        config.initial_balances = [];
        config.mint = {
            minter: artifact.bbro_minter,
        };
        return config;
    }
}

// rewards pool
export class RewardsPool implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: RewardsPoolConfig;
    public address: string;
    private updateOwner: string; // store owner from initial config to move ownership via update function

    constructor(client: TerraClient, config: RewardsPoolConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_rewards_pool.wasm";
        this.updateOwner = config.owner;
        this.instantiateMsg = this.setInstantiateMsg(config, artifact);
        this.address = artifact.rewards_pool;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.rewards_pool = this.address;
    }

    public async addDistributor(address: string): Promise<void> {
        await this.client.executeContract(
            this.address,
            {
                add_distributor: {
                    distributor: address,
                }
            }
        )
    }

    public async moveOwnership(): Promise<void> {
        await this.client.executeContract(
            this.address,
            {
                update_config: {
                    owner: this.updateOwner,
                }
            }
        );
    }

    private setInstantiateMsg(config: RewardsPoolConfig, artifact: Artifact): RewardsPoolConfig {
        config.owner = this.client.wallet.key.accAddress;
        config.bro_token = artifact.bro_token;
        return config;
    }
}

// mvp treasury
export class Treasury implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: TreasuryConfig;
    public address: string;

    constructor(client: TerraClient, config: TreasuryConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_mvp_treasury.wasm";
        this.instantiateMsg = config;
        this.address = artifact.mvp_treasury;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.mvp_treasury = this.address;
    }
}

// epoch manager
export class EpochManager implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: EpochManagerConfig;
    public address: string;

    constructor(client: TerraClient, config: EpochManagerConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_epoch_manager.wasm";
        this.instantiateMsg = config;
        this.address = artifact.epoch_manager;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.epoch_manager = this.address;
    }
}

// staking v1
export class StakingV1 implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: StakingV1Config;
    public address: string;

    constructor(client: TerraClient, config: StakingV1Config, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_staking_v1.wasm";
        this.instantiateMsg = this.setInstantiateMsg(config, artifact);
        this.address = artifact.staking_v1;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.staking_v1 = this.address;
    }

    private setInstantiateMsg(config: StakingV1Config, artifact: Artifact): StakingV1Config {
        config.bro_token = artifact.bro_token;
        config.rewards_pool_contract = artifact.rewards_pool;
        config.bbro_minter_contract = artifact.bbro_minter;
        config.epoch_manager_contract = artifact.epoch_manager;
        return config;
    }
}

// bonding v1
export class BondingV1 implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: BondingV1Config;
    public address: string;

    constructor(client: TerraClient, config: BondingV1Config, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_bonding_v1.wasm";
        this.instantiateMsg = this.setInstantiateMsg(config, artifact);
        this.address = artifact.bonding_v1;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.bonding_v1 = this.address;
    }

    private setInstantiateMsg(config: BondingV1Config, artifact: Artifact): BondingV1Config {
        config.bro_token = artifact.bro_token;
        config.treasury_contract = artifact.mvp_treasury;
        return config;
    }
}

// distributor v1
export class DistributorV1 implements Contract {
    public client: TerraClient;
    public artifact: string;
    public instantiateMsg: DistributorV1Config;
    public address: string;

    constructor(client: TerraClient, config: DistributorV1Config, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_distributor_v1.wasm";
        this.instantiateMsg = this.setInstantiateMsg(config, artifact);
        this.address = artifact.distributor_v1;
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.distributor_v1 = this.address;
    }

    private setInstantiateMsg(config: DistributorV1Config, artifact: Artifact): DistributorV1Config {
        config.epoch_manager_contract = artifact.epoch_manager;
        config.rewards_contract = artifact.rewards_pool;
        config.staking_contract = artifact.staking_v1;
        config.bonding_contract = artifact.bonding_v1;
        return config;
    }
}
