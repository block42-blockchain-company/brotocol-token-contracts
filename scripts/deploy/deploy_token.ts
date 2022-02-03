import dotenv from 'dotenv';
import { loadConfig } from './lib/config.js';
import { loadArtifact } from './lib/artifact.js';
import { TerraClient } from './lib/client.js';
import { BroToken, deployContract } from './lib/contracts.js';

async function main() {
    dotenv.config();

    const chainID = String(process.env.CHAINID);
    const admin = String(process.env.ADMIN_ADDRESS);

    const artifact = loadArtifact(chainID);
    const config = loadConfig(chainID);
    const terraClient = new TerraClient();

    // set artifact network
    artifact.network = chainID;

    if (!config.deployToken) {
        console.log(`Token deploy function is disabled. Current BRO token address:\n${artifact.bbro_token}`);
        return;
    }

    // Deploy BRO token
    const broTokenContract = new BroToken(terraClient, config.bro_token, config.initialBroBalanceHolderAddress, artifact);
    await deployContract(chainID, artifact, broTokenContract, admin);
}

main().catch(console.log);
