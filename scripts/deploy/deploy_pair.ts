import dotenv from 'dotenv';
import { loadConfig } from './lib/config.js';
import { loadArtifact, writeArtifact } from './lib/artifact.js';
import { TerraClient } from './lib/client.js';
import { AstroFactory, deployContract, Oracle } from './lib/contracts.js';

async function main() {
    dotenv.config();

    const chainID = String(process.env.CHAINID);
    const admin = String(process.env.ADMIN_ADDRESS);

    const artifact = loadArtifact(chainID);
    const config = loadConfig(chainID);
    const terraClient = new TerraClient();

    // set artifact network
    artifact.network = chainID;

    if (!artifact.bro_token) {
        throw Error("BRO token address must be stored in artifact. Deploy token first using deploy_token.ts script.");
    }

    // Create BRO/UST pair on astroport
    if (config.bro_ust_pair.createPair) {
        if (!config.bro_ust_pair.factory_address) {
            throw Error("Specify astro-factory address to create a new pair contract");
        }

        const astroFactory = new AstroFactory(terraClient, config.bro_ust_pair);
        await astroFactory.createBroUstPair(artifact);

        console.log(`BRO/UST pair created. Pair address: ${artifact.bro_ust_pair}`);
    } else {
        console.log(`BRO/UST pair deploy disabled. Current pair address: ${artifact.bro_ust_pair}`);
    }

    // set bro_ust lp token address
    const poolInfo = await terraClient.queryContract<any>(artifact.bro_ust_pair, { pair: {} });
    artifact.bro_ust_lp_token = poolInfo.liquidity_token;
    writeArtifact(artifact, chainID);
    console.log(`bro/ust lp token stored. Contract address: ${artifact.bro_ust_lp_token}`);

    // Deploy oracle
    const oracleContract = new Oracle(terraClient, config, artifact);
    await deployContract(chainID, artifact, oracleContract, admin);
}

main().catch(console.log);
