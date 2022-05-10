# Bonding V1

The Bonding Contract contains logic for buying BRO token by discounted price by providing UST or UST/BRO LP Tokens from Astroport.
Price calculation will depend on current BRO market price.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "bro_token": "terra1...",
    "rewards_pool_contract": "terra1...",
    "treasury_contract": "terra1...",
    "astroport_factory": "terra1...",
    "oracle_contract": "terra1...",
    "ust_bonding_discount": "0.05",
    "min_bro_payout": "100",
    "bonding_mode": {
        "normal": {
            "ust_bonding_reward_ratio": "0.6",
            "lp_token": "terra1...",
            "lp_bonding_discount": "0.05",
            "vesting_period_blocks": 50
        },
        |
        "community": {
            "staking_contract": "terra1...",
            "epochs_locked": 1
        }
    }
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

Claim available reward amount.

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
        "lp_token": "terra1...",
        "rewards_pool_contract": "terra1...",
        "treasury_contract": "terra1...",
        "astroport_factory": "terra1...",
        "oracle_contract": "terra1...",
        "ust_bonding_discount": "0.05",
        "min_bro_payout": "100",
    }
}
```

### `update_bonding_mode_config`
Updates specific settings for bonding mode config. Can be executed only by owner.
Message params are optional.

```json
{
    "update_bonding_mode_config": {
        "ust_bonding_reward_ratio_normal": "0.1",
        "lp_token_normal": "terra1...",
        "lp_bonding_discount_normal": "0.1",
        "vesting_period_blocks_normal": 100,
        "staking_contract_community": "terra1...",
        "epochs_locked_community": 100
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

### `simulate_ust_bond`

Returns simulated bro bond using specified uusd amount.

```json
{
    "simulate_ust_bond": {
        "uusd_amount": "100"
    }
}
```

### `simulate_lp_bond`

Returns simulated bro bond using specified ust/bro lp token amount.
Disabled for `BondingMode::Community` mode.

```json
{
    "simulate_lp_bond": {
        "lp_amount": "100"
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
{
    "bonding_mode": {
        "normal": {
            "ust_bonding_reward_ratio": "0.5",
            "lp_token": "terra1...",
            "lp_bonding_discount": "0.05",
            "vesting_period_blocks": 10
        }
    }
}
```
