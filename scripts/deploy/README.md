# Deploy scripts

### Requirements
For local deploy: 
1) deploy astroport
2) specify astro factory address in config

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

For local development this will be enough
```env
CHAINID=localterra
ADMIN_ADDRESS="terra1..."
```

### 2. Install dependencies
```
npm install
```

### 3. Edit .json config file for preferred chain

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
deploy-pair
```

Deploy IDO related contracts
```
npm run deploy-core
```

### 5. Check ./artifacts folder for deployed contract addresses
