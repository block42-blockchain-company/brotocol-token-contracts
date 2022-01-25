# MVP Treasury

The mvp-treasury contract is used for holding assets that were exchanged to BRO token
via bonding.

---

## InstantiateMsg

```json
{
    "owner": "terra1..."
}
```

## ExecuteMsg

### `spend`

Sends whole treasury balance of specified asset to recipient.
Can be executed only by owner.

```json
{
    "asset_info": {
        "native": {
            "denom": "uusd"
        }
    },
    "recipient": "terra1..."
}
```

## QueryMsg

### `config`

Returns mvp-treasury contract config.

```json
{
    "config": {}
}
```

### `balance`

Returns mvp-treasury contract balance of specified asset.

```json
{
    "balance": {
        "asset_info": {
            "native": {
                "denom": "uusd"
            }
        }
    }
}
```
