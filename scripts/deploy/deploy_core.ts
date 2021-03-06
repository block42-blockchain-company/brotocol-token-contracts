import dotenv from 'dotenv'
import { loadConfig } from './lib/config.js';
import { loadArtifact } from './lib/artifact.js';
import { TerraClient } from './lib/client.js';
import { 
    Airdrop,
    BbroMinter,
    BbroToken,
    BondingV1,
    deployContract,
    DistributorV1,
    EpochManager,
    IdoTreasury,
    OpReserveTreasury,
    RewardsPool,
    StakingV1,
    TokenPool,
    MvpTreasury,
    Vesting,
    WhitelistSale
} from './lib/contracts.js';
import { sleep } from './lib/util.js';

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
    console.log("Update bbro-token address for bbro-minter success");

    // Deploy rewards pool
    const rewardsPoolContract = new RewardsPool(terraClient, config.rewards, artifact);
    await deployContract(chainID, artifact, rewardsPoolContract, admin);

    // Deploy mvp treasury
    const treasuryContract = new MvpTreasury(terraClient, config.treasury, artifact);
    await deployContract(chainID, artifact, treasuryContract, admin);

    // Deploy ido treasury
    const idoTreasuryContract = new IdoTreasury(terraClient, config.ido_treasury, artifact);
    await deployContract(chainID, artifact, idoTreasuryContract, admin);

    // Deploy op reserve treasury
    const opReserveTreasury = new OpReserveTreasury(terraClient, config.op_reserve_treasury, artifact);
    await deployContract(chainID, artifact, opReserveTreasury, admin);

    // Deploy token pool
    const tokenPoolContract = new TokenPool(terraClient, config.token_pool, artifact);
    await deployContract(chainID, artifact, tokenPoolContract, admin);

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
    await sleep(5000);

    console.log("propose ownership of rewards pool to configured owner\nclaim it from owner address");
    await rewardsPoolContract.proposeConfiguredOwner();
    await sleep(5000);

    console.log("whitelist staking contract in bbro-minter");
    await bbroMinterContract.addMinter(artifact.staking_v1);
    await sleep(5000);

    console.log("propose ownership of bbro-minter to configured owner\nclaim it from owner address");
    await bbroMinterContract.proposeConfiguredOwner();
    await sleep(5000);

    console.log(`You can find deployed contract addresses in artifacts folder: artifacts/${chainID}.json`);
}

main().catch(console.log);
