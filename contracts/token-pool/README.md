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

Returns token pool contract config.

```json
{
    "config": {}
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
