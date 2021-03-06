# Whitelist Sale

The Whitelist Sale contract contains logic for selling BRO token by fixed price for whitelisted accounts e.g. Brotocol NFT Holders.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "bro_token": "terra1...",
    "bro_amount_per_uusd": "10",
    "bro_amount_per_nft": "2",
    "ust_receiver": "terra1...",
    "rewards_pool_contract": "terra1..."
}
```

## ExecuteMsg

### `receive`

Receives a hook message and processes it depending on the received template.

```json
{
    "receive": {
        "sender": "terra1...",
        "amount": "100",
        "msg": "<base64_encoded_json_string>"
    }
}
```

### `receive:register_sale`

Registers sale. Can be executed only by owner.

```json
{
    "sale_start_time": 100,
    "sale_end_time": 200
}
```

### `register_accounts`

Registers a list of accounts for sale. Can be executed only by owner.

```json
{
    "register_accounts": {
        "accounts": [
            {
                "address": "terra1...",
                "owned_nfts_count": 1
            }
        ]
    }
}
```

### `purchase`

Purchase bro by fixed price by providing ust amount.

```json
{
    "purchase": {}
}
```

### `withdraw_remaining_balance`

Withdraw remaining bro balance after sale is over.

```json
{
    "withdraw_remaining_balance": {}
}
```

## QueryMsg

### `config`

Returns whitelist sale contract config.

```json
{
    "config": {}
}
```

### `state`

Returns whitelist sale contract state.

```json
{
    "state": {}
}
```

### `whitelisted_account`

Returns whitelisted account info by specified address.

```json
{
    "whitelisted_account": {
        "address": "terra1..."
    }
}
```

## MigrateMsg

```json
{}
```
