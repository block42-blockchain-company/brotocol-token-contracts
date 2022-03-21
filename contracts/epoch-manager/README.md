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
