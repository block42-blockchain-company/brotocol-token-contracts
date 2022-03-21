# Brotocol Oracle

The oracle contract performs calculation x*y=k pair assets average prices based on accumulations and time period.

---

## InstantiateMsg

```json
{
  "factory_contract": "terra...",
  "asset_infos": [
    {
      "token": {
        "contract_addr": "terra..."
      }
    },
    {
      "native_token": {
        "denom": "uusd"
      }
    }
  ],
  "price_update_interval": 86400
}
```

## ExecuteMsg

### `update_config`

Updates oracle contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "price_update_interval": 86400
}
```

### `update_price`

Updates pair average and cumulative prices.

```json
{
    "update_price": {}
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

Returns oracle contract config.

```json
{
    "config": {}
}
```

### `consult_price`

Converts specified asset amount into other using average price.

```json
{
  "consult": {
    "token": {
      "native_token": {
        "denom": "uusd"
      }
    },
    "amount": "123"
  }
}
```

### `is_ready_to_trigger`

Returns whether oracle can be updated or not

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
