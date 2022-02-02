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
    "owner": "terra1...",
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

## QueryMsg

### `config`

Returns oracle contract config.

```json
{
    "config": {}
}
```

### `consult_price`

Multiplies a given amount and last average price in common.

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

## MigrateMsg

```json
{}
```
