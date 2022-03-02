# Deploy scripts

### Requirements
For local deploy: 
1) deploy astroport
2) specify astro factory address in config

*When deploying astroport, and it fails, try specifying a `multisigAddress` in `artifacts/localterra.json.*

*Will work with this dummy address `"multisigAddress": "terra1c7m6j8ya58a2fkkptn8fgudx8sqjqvc8azq0ex"`*

For testnet/mainnet:
1) specify astro factory address in config

### 1. Create .env using .env.defaults template
Example
```env
MNEMONIC="mnemonic"
LCD="http://localhost:1317"
CHAINID=localterra
ADMIN_ADDRESS="terra1..."
```

For local development this will be enough, you can use any address for `ADMIN_ADDRESS`
```env
CHAINID=localterra
ADMIN_ADDRESS="terra1..."
```

### 2. Install dependencies
```
npm install
```

### 3. Create .json config file in ./config dir
Config template can be found in config/template.json.
For example if you want to deploy contracts to the localterra you need to create file with following name `localterra.json` and fill all the params with preffered data.

Supported configs: `localterra.json` (local), `bombay-12.json` (testnet), `columbus-5.json` (mainnet).

## Deploy

### Deploy IDO
```
npm run deploy-ido
```

### Separate deployment
Deploy BRO token
```
npm run deploy-token
```

Create BRO/UST pair and deploy oracle
```
npm run deploy-pair
```

Deploy IDO related contracts
```
npm run deploy-core
```

### 5. Check ./artifacts folder for deployed contract addresses
