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

### `claim`

Claims available amount for message sender at specified airdrop round.

```json
{
    "claim": {
        "stage": 1,
        "amount": "100",
        "proof": [
            "<keccak256_string>",
            "<keccak256_string>"
        ]
    }
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