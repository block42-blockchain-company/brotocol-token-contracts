# Distributor

The distributor contract is used for token distribution for staking rewards
and bonding.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "distribution_genesis_block": 12500,
    "epoch_manager_contract": "terra1...",
    "rewards_contract": "terra1...",
    "staking_contract": "terra1...",
    "staking_distribution_amount": "100",
    "bonding_contract": "terra1...",
    "bonding_distribution_amount": "100"
}
```

## ExecuteMsg

### `distribute`

Performs token distribution for staking and bonding.

```json
{
    "distribute": {}
}
```

### `update_config`

Updates distributor contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "paused": false,
    "distribution_genesis_block": 12500,
    "epoch_manager_contract": "terra1...",
    "rewards_contract": "terra1...",
    "staking_contract": "terra1...",
    "staking_distribution_amount": "100",
    "bonding_contract": "terra1...",
    "bonding_distribution_amount": "100"
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

Returns dsitributor contract config.

```json
{
    "config": {}
}
```

### `last_distribution`

Returns information about last distribution.

```json
{
    "last_distribution": {}
}
```

### `is_ready_to_trigger`

Returns whether funds can be distributed or not

```json
{
    "is_ready_to_trigger": {}
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
