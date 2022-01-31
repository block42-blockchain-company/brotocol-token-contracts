# Epoch Manager

The epoch-manager contract is used for synchronizing global information for brotocol
such as epoch info, blocks per year, bbro emission rate and etc.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "epoch": 10,
    "blocks_per_year": 1000,
    "bbro_emission_rate": "1.0"
}
```

## ExecuteMsg

### `update_config`

Updates contract settings. Can be executed only by owner.
Message params are optional.

```json
{
    "owner": "terra1..."
}
```

### `update_state`

Updates contract state. Can be executed only by owner.
Message params are optional.

```json
{
    "epoch": 11,
    "blocks_per_year": 1230,
    "bbro_emission_rate": "0.9"
}
```

## QueryMsg

### `config`

Returns epoch-manager contract config.

```json
{
    "config": {}
}
```

### `epoch_info`

Returns epoch-manager contract state.

```json
{
    "epoch_info": {}
}
```

## MigrateMsg

```json
{}
```
