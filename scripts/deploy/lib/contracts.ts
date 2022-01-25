import { AirdropConfig, BroTokenConfig, VestingConfig } from "./config.js";
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

    constructor(client: TerraClient, broConfig: BroTokenConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "cw20_base.wasm";
        this.instantiateMsg = this.setInstantiateMsg(broConfig);
        this.address = "";
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.bro_token = this.address;
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
    
    constructor(client: TerraClient, airdropConfig: AirdropConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_airdrop.wasm";
        this.instantiateMsg = this.setInstantiateMsg(airdropConfig, artifact);
        this.address = "";
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

    constructor(client: TerraClient, vestingConfig: VestingConfig, artifact: Artifact) {
        this.client = client;
        this.artifact = "brotocol_vesting.wasm";
        this.instantiateMsg = this.setInstantiateMsg(vestingConfig, artifact);
        this.address = "";
    }

    public setArtifactData(artifact: Artifact): void {
        artifact.vesting = this.address;
    }

    private setInstantiateMsg(config: VestingConfig, artifact: Artifact): VestingConfig {
        config.bro_token = artifact.bro_token;
        return config;
    }
}