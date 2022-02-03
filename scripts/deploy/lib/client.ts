import { 
    BlockTxBroadcastResult,
    Coins,
    isTxError,
    LCDClient,
    LocalTerra, MnemonicKey, MsgExecuteContract, MsgInstantiateContract, MsgStoreCode, Wallet
} from "@terra-money/terra.js";
import * as fs from 'fs';
import path from 'path'
import { sleep } from "./util.js";

const ARTIFACTS_BASE_PATH = "../../artifacts";
const TIMEOUT = 3000;

export class TerraClient {
    public wallet: Wallet;
    public terra: LCDClient | LocalTerra;

    constructor() {
        if (process.env.MNEMONIC) {
            this.terra = new LCDClient({
                URL: String(process.env.LCD),
                chainID: String(process.env.CHAINID),
            });
            this.wallet = this.terra.wallet(new MnemonicKey({
                mnemonic: process.env.MNEMONIC,
            }));
        } else {
            const localTerra = new LocalTerra();
            this.terra = localTerra;
            this.wallet = this.terra.wallet(new MnemonicKey({
                mnemonic: "satisfy adjust timber high purchase tuition stool faith fine install that you unaware feed domain license impose boss human eager hat rent enjoy dawn",
            }));
        }
    }

    public async storeCode(artifactName: string): Promise<number> {
        const storeCodeMsg = new MsgStoreCode(
            this.wallet.key.accAddress,
            fs.readFileSync(path.join(ARTIFACTS_BASE_PATH, artifactName)).toString('base64'),
        );

        const storeCodeTx = await this.wallet.createAndSignTx({
            msgs: [storeCodeMsg],
        });

        const txResponse = await this.terra.tx.broadcast(storeCodeTx);
        sleep(TIMEOUT);
        
        this.throwOnError(txResponse);

        const codeID = Number((txResponse as any).logs[0].eventsByType.store_code.code_id[0]);
        console.log(`${artifactName} store code success. code_id: ${codeID}`);

        return codeID;
    }

    public async instantiateContract(
        adminAddress: string,
        codeID: number,
        msg: object,
        coins: Coins.Input | undefined,
    ): Promise<string> {
        const instantiateMsg = new MsgInstantiateContract(
            this.wallet.key.accAddress,
            adminAddress,
            codeID,
            msg,
            coins,
        );

        const instantiateTx = await this.wallet.createAndSignTx({
            msgs: [instantiateMsg],
        });

        const txResponse = await this.terra.tx.broadcast(instantiateTx);
        sleep(TIMEOUT);

        this.throwOnError(txResponse);

        const address = String(txResponse.logs[0].events[0].attributes.filter(element => element.key == 'contract_address' ).map(x => x.value).shift());
        console.log(`Instantiate contract with code_id ${codeID} success. Contract address:\n${address}`);
        console.log(""); // empty line

        return address;
    }

    public async queryContract<T>(contractAddress: string, query: object): Promise<T> {
        return await this.terra.wasm.contractQuery<T>(contractAddress, query);
    }

    public async executeContract(contractAddress: string, msg: object, coins?: Coins.Input): Promise<BlockTxBroadcastResult> {
        const executeMsg = new MsgExecuteContract(
            this.wallet.key.accAddress,
            contractAddress,
            msg,
            coins,
        );

        const execuiteTx = await this.wallet.createAndSignTx({
            msgs: [executeMsg],
        });

        const txResponse = await this.terra.tx.broadcast(execuiteTx);
        sleep(TIMEOUT);

        this.throwOnError(txResponse);
        return txResponse;
    }

    private throwOnError(tx: any) {
        if (isTxError(tx)) {
            throw new Error(
                `store code failed. code: ${tx.code}, codespace: ${tx.codespace}, raw_log: ${tx.raw_log}`
            );
        }
    }
}
