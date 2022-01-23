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

### `update_config`

Updates airdrop contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "owner": "terra1...",
    "bro_token": "terra1..."
}
```

### `register_merkleRoot`

Registers merkle root hash. Can be executed only by owner.

```json
{
    "merkle_root": "hd1082hd01dj12j2d12"
}
```

### `claim`

Claims availalble amount for message sender at specified airdrop round.

```json
{
    "stage": 1,
    "amount": "100",
    "proof": [
        "djawd92jd91j",
        "klf92h038fhb"
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
