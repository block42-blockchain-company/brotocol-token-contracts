# bBro minter

The bBro-minter contract is used for minting and burning bBro tokens.
This contract is set as a minter for bBro token.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "whitelist": [
        "terra1..."
    ]
}
```

## ExecuteMsg

### `update_config`

Updates bbro-minter contract config. Can be executed only by owner.
Message params are optional.

```json
{
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

### `propose_new_owner`

Creates an offer for a new owner. Only owner can execute this function.

```json
{
    "propose_new_owner": {
        "new_owner": "terra1...",
        "expires_in_blocks": 100
    }
}
```

### `drop_ownership_proposal`

Removes the existing offer for the new owner. Only owner can execute this function

```json
{
    "drop_ownership_proposal": {}
}
```

### `claim_ownership`

Used to claim(approve) new owner proposal, thus changing contract's owner.
Only address proposed as a new owner can execute this function.

```json
{
    "claim_ownership": {}
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

### `ownership_proposal`

Returns information about created ownership proposal otherwise returns not-found error.

```json
{
    "ownership_proposal": {}
}

## MigrateMsg

```json
{}
```
