# Token pool

The Token pool Contract holds the funds for distributions.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "bro_token": "terra1..."
}
```

## ExecuteMsg

### `transfer`

Transfer specified amount to specified address. Can be executed only by owner.

```json
{
    "transfer": {
        "recipient": "terra1...",
        "amount": "100"
    }
}
```

### `send`

Transfer specified amount to specified contract with provided execute msg. Can be executed only by owner.

```json
{
    "send": {
        "contract": "terra1...",
        "amount": "100",
        "msg": "<base64_encoded_json_string>"
    }
}
```

### `update_config`

Updates bonding contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "update_config": {
        "owner": "terra1..."
    }
}
```

## QueryMsg

### `config`

Returns token pool contract config.

```json
{
    "config": {}
}
```

## MigrateMsg

```json
{}
```
