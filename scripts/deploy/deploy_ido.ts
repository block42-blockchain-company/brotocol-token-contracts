import dotenv from 'dotenv'
import { loadConfig } from './lib/config.js';
import { TerraClient } from './lib/client.js';
import { Airdrop, BroToken, deployContract, Vesting } from './lib/contracts.js';
import { loadArtifact } from './lib/artifact.js';

async function main() {
    dotenv.config();
    
    const chainID = String(process.env.CHAINID);
    const admin = String(process.env.ADMIN_ADDRESS);

    const artifact = loadArtifact(chainID);
    const config = loadConfig(chainID);
    const terraClient = new TerraClient();

    // set artifact network
    artifact.network = chainID;

    // Deploy BRO token
    const broContract = new BroToken(terraClient, config.bro_token, artifact);
    await deployContract(chainID, artifact, broContract, admin);

    // Deploy airdrop
    const airdropContract = new Airdrop(terraClient, config.airdrop, artifact);
    await deployContract(chainID, artifact, airdropContract, admin);

    // Deploy vesting
    const vestingContract = new Vesting(terraClient, config.vesting, artifact);
    await deployContract(chainID, artifact, vestingContract, admin);

    // Deploy bbro-minter

    console.log(`You can find deployed contract addresses in artifacts folder: artifacts/${chainID}.json`);
}

main().catch(console.log);