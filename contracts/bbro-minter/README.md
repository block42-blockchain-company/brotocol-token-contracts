# bBro minter

The bBro-minter contract is used for minting and burning bBro tokens.
This contract is set as a minter for bBro token.

---

## InstantiateMsg

```json
{
    "gov_contract": "terra1...",
    "whitelist": [
        "terra1..."
    ]
}
```

## ExecuteMsg

### `instantiate_token`

Creates new token contract using code_id and specified Cw20Instantiate msg.
Can be executed only by owner.

```json
{
    "code_id": 1,
    "token_instantiate_msg": {
        "name": "bBro token",
        "symbol": "bBRO",
        "decimals": 6,
        "initial_balances": [],
        "mint": null,
        "marketing": null,
    }
}
```

### `update_config`

Updates bbro-minter contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "new_gov_contract": "terra1...",
    "bbro_token": "terra1..."
}
```

### `add_minter`

Adds new minter address into whitelist. Can be executed only by owner.

```json
{
    "add_minter": {
        "minter": "terra1..."
    }
}
```

### `remove_minter`

Removes minter from whitelist. Can be executed only by owner.

```json
{
    "remove_minter": {
        "minter": "terra1..."
    }
}
```

### `mint`

Mints specified amount for specified address.
Can be executed only by whitelisted address.

```json
{
    "mint": {
        "recipient": "terra1...",
        "amount": "100"
    }
}
```

### `burn`

Burns specified amount from specified address balance.
Can be executed only by whitelisted address.

```json
{
    "burn": {
        "owner": "terra1...",
        "amount": "100"
    }
}
```

## QueryMsg

### `config`

Returns bbro-minter contract config.

```json
{
    "config": {}
}
```
