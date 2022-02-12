# Airdrop

The Airdrop contract is used for airdropping BRO tokens to TBA.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "bro_token": "terra1..."
}
```

## ExecuteMsg

### `receive`

Receives a hook message and processes it depending on the received template.

```json
{
    "receive": {
        "sender": "terra1...",
        "amount": "100",
        "msg": "<base64_encoded_json_string>"
    }
}
```

### `receive:register_merkle_root`

Registers merkle root hash. Can be executed only by owner.

```json
{
    "merkle_root": "<keccak256_string>"
}
```

### `update_config`

Updates airdrop contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "owner": "terra1..."
}
```

### `claim`

Claims availalble amount for message sender at specified airdrop round.

```json
{
    "stage": 1,
    "amount": "100",
    "proof": [
        "<keccak256_string>",
        "<keccak256_string>"
    ]
}
```

## QueryMsg

### `config`

Returns airdrop contract config.

```json
{
    "config": {}
}
```

### `latest_stage`

Returns the number of latest stage.

```json
{
    "latest_stage": {}
}
```

### `merkle_root`

Returns merkle root information by specified stage.

```json
{
    "merkle_root": {
        "stage": 1
    }
}
```

### `is_claimed`

Returns claim information by specified stage and address.

```json
{
    "is_claimed": {
        "stage": 1,
        "address": "terra1..."
    }
}
```

## MigrateMsg

```json
{}
```