# Rewards Pool

The Rewards pool contract holds the funds which will be distributed as a rewards.
Right now for staking and bonding.

---
## InstantiateMsg

```json
{
    "owner": "terra1...",
    "bro_token": "terra1...",
    "spend_limit": "1000",
    "whitelist": [
        "terra1..."
    ]
}
```

## ExecuteMsg

### `update_config`

Updates rewards pool contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "spend_limit": "1001"
}
```

### `add_distributor`

Adds new distributor address into whitelist. Can be executed only by owner.

```json
{
    "add_distributor": {
        "distributor": "terra1..."
    }
}
```

### `remove_distributor`

Removes distributor from whitelist. Can be executed only by owner.

```json
{
    "remove_distributor": {
        "distributor": "terra1..."
    }
}
```

### `distribute_rewards`

Distributes rewards to specified contracts.
Can be executed only by whitelisted address.

```json
{
    "distribute_rewards": {
        "distributions": [
            {
                "contract": "terra1...",
                "amount": "100",
                "msg": "<base64_encoded_json_string>"
            }
        ]
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

Returns rewards pool contract config.

```json
{
    "config": {}
}
```

### `balance`

Returns rewards pool BRO token balance.

```json
{
    "balance": {}
}
```

### `ownership_proposal`

Returns information about created ownership proposal otherwise returns not-found error.

```json
{
    "ownership_proposal": {}
}
```

## MigrateMsg

```json
{}
```
