import dotenv from 'dotenv'
import { loadConfig } from './lib/config.js';
import { loadArtifact } from './lib/artifact.js';
import { TerraClient } from './lib/client.js';
import { Airdrop, BbroMinter, BbroToken, BondingV1, BroToken, deployContract, DistributorV1, EpochManager, RewardsPool, StakingV1, Treasury, Vesting, WhitelistSale } from './lib/contracts.js';

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

    if (!artifact.oracle) {
        throw Error("Price oracle for BRO/UST pair must be stored in artifact. Deploy pair and oracle first using deploy_pair.ts script");
    }

    // Deploy airdrop
    const airdropContract = new Airdrop(terraClient, config.airdrop, artifact);
    await deployContract(chainID, artifact, airdropContract, admin);

    // Deploy vesting
    const vestingContract = new Vesting(terraClient, config.vesting, artifact);
    await deployContract(chainID, artifact, vestingContract, admin);

    // Deploy bbro-minter
    const bbroMinterContract = new BbroMinter(terraClient, config.bbro_minter, artifact);
    await deployContract(chainID, artifact, bbroMinterContract, admin);

    // Deploy bbro-token
    const bbroTokenContract = new BbroToken(terraClient, config.bbro_token, artifact);
    await deployContract(chainID, artifact, bbroTokenContract, admin);

    // update bbro-token address in bbro-minter
    await bbroMinterContract.updateConfig(artifact.bbro_token);
    console.log("Update owner and bbro-token address for bbro-minter success");

    // Deploy rewards pool
    const rewardsPoolContract = new RewardsPool(terraClient, config.rewards, artifact);
    await deployContract(chainID, artifact, rewardsPoolContract, admin);

    // Deploy mvp treasury
    const treasuryContract = new Treasury(terraClient, config.treasury, artifact);
    await deployContract(chainID, artifact, treasuryContract, admin);

    // Deploy epoch manager
    const epochManagerContract = new EpochManager(terraClient, config.epoch_manager, artifact);
    await deployContract(chainID, artifact, epochManagerContract, admin);

    // Deploy staking
    const stakingContract = new StakingV1(terraClient, config.stakingv1, artifact);
    await deployContract(chainID, artifact, stakingContract, admin);

    // Deploy bonding
    const bondingContract = new BondingV1(terraClient, config.bondingv1, artifact);
    await deployContract(chainID, artifact, bondingContract, admin);

    // Deploy whitelist sale
    const whitelistSaleContract = new WhitelistSale(terraClient, config.whitelist_sale, artifact);
    await deployContract(chainID, artifact, whitelistSaleContract, admin);

    // Deploy distributor
    const distributorContract = new DistributorV1(terraClient, config.distributorv1, artifact);
    await deployContract(chainID, artifact, distributorContract, admin);

    console.log("whitelist distributor in rewards pool");
    await rewardsPoolContract.addDistributor(artifact.distributor_v1);

    console.log("move ownership of rewards pool to configured owner");
    await rewardsPoolContract.moveOwnership();

    console.log("whitelist staking contract in bbro-minter");
    await bbroMinterContract.addMinter(artifact.staking_v1);

    console.log("move ownership of bbro-minter to configured owner");
    await bbroMinterContract.moveOwnership();

    console.log("distribute bro tokens to contracts");
    const broTokenContract = new BroToken(terraClient, config.bro_token, config.initialBroBalanceHolderAddress, artifact);
    await broTokenContract.transfer(artifact.vesting, config.bro_distributions.vesting);
    await broTokenContract.transfer(artifact.rewards_pool, config.bro_distributions.rewards);

    console.log(`You can find deployed contract addresses in artifacts folder: artifacts/${chainID}.json`);
}

main().catch(console.log);
