# Deploy scripts

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

4.1 Deploy IDO
```
npm run deploy-ido
```

### 5. Check ./artifacts folder for deployed contract addresses
