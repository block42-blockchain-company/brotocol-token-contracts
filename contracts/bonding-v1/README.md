# Bonding V1

The Bonding Contract contains logic for buying BRO token by discounted price by providing UST or UST/BRO LP Tokens from Astroport.
Price calculation will depend on current BRO market price.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "bro_token": "terra1...",
    "lp_token": "terra1...",
    "treasury_contract": "terra1...",
    "astroport_factory": "terra1...",
    "ust_bonding_reward_ratio": "0.6",
    "ust_bonding_discount": "0.05",
    "lp_bonding_discount": "0.05",
    "min_bro_payout": "100",
    "vesting_period_blocks": 50
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

### `receive:distribute_reward`

Distributes received reward.

```json
{
    "distribute_reward": {}
}
```

### `receive:lp_bond`

Bond bro tokens by providing lp token amount.

```json
{
    "lp_bond": {}
}
```

### `ust_bond`

Bond bro tokens by providing ust amount.

```json
{
    "ust_bond": {}
}
```

### `claim`

Claim availalble reward amount.

```json
{
    "claim": {}
}
```

### `update_config`

Updates bonding contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "update_config": {
        "owner": "terra1...",
        "lp_token": "terra1...",
        "treasury_contract": "terra1...",
        "astroport_factory": "terra1...",
        "ust_bonding_reward_ratio": "0.6",
        "ust_bonding_discount": "0.05",
        "lp_bonding_discount": "0.05",
        "min_bro_payout": "100",
        "vesting_period_blocks": 50
    }
}
```

## QueryMsg

### `config`

Returns bonding contract config.

```json
{
    "config": {}
}
```

### `state`

Returns bonding contract state.

```json
{
    "state": {}
}
```

### `claims`

Returns available claims for bonder by specified address.

```json
{
    "claims": {
        "address": "terra1..."
    }
}
```

## MigrateMsg

```json
{}
```