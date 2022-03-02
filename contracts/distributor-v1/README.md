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
    "owner": "terra1...",
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
## MigrateMsg

```json
{}
```
