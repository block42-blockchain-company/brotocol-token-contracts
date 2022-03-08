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
```

## MigrateMsg

```json
{}
```
