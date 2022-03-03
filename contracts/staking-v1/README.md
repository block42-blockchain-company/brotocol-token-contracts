# Staking V1

The Staking Contract contains the logic for BRO Token staking and reward distribution.
Also bBRO tokens will be minted as a reward for staking BRO.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "bro_token": "terra1...",
    "rewards_pool_contract": "terra1...",
    "bbro_minter_contract": "terra1...",
    "epoch_manager_contract": "terra1...",
    "unstake_period_blocks": 100,
    "min_staking_amount": "100",
    "min_lockup_period_epochs": 1,
    "max_lockup_period_epochs": 10,
    "base_rate": "0.005",
    "linear_growth": "0.0001",
    "exponential_growth": "0.000075"
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
    "distribute_reward": {
        "distributed_at_block": 12350
    }
}
```

### `receive:stake`

Deposits specified amount of tokens to get reward shares.

```json
{
    "stake": {
        "stake_type": {
            "unlocked": {}
        }
    }
}
```

### `lockup_staked`

Lockup unlocked staked amount.

```json
{
    "lockup_staked": {
        "amount": "100",
        "epochs_locked": 10
    }
}
```

### `unstake`

Unstake staked amount of tokens. Tokens will be claimable only after passing unstaking period.

```json
{
    "unstake": {
        "amount": "100"
    }
}
```

### `withdraw`

Withdraw the amount of tokens that have already passed the unstaking period.

```json
{
    "withdraw": {}
}
```

### `claim_bro_rewards`

Claim available bro reward amount.

```json
{
    "claim_bro_rewards": {}
}
```

### `claim_bbro_rewards`

Claim available bbro reward amount.

```json
{
    "claim_bbro_rewards": {}
}
```

### `update_config`

Updates staking contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "owner": "terra1...",
    "paused": false,
    "unstake_period_blocks": 1,
    "min_staking_amount": "100",
    "min_lockup_period_epochs": 1,
    "max_lockup_period_epochs": 10,
    "base_rate": "0.1",
    "linear_growth": "0.2",
    "exponential_growth": "0.3"
}
```

## QueryMsg

### `config`

Returns staking contract config.

```json
{
    "config": {}
}
```

### `state`

Returns staking contract state.

```json
{
    "state": {}
}
```

### `staker_info`

Returns staker info by specified address.

```json
{
    "staker_info": {
        "staker": "terra1..."
    }
}
```

### `staker_accrued_rewards`

Returns available amount for staker to claim by specified address.

```json
{
    "staker_accrued_rewards": {
        "staker": "terra1..."
    }
}
```

### `withdrawals`

Returns available withdrawals for staker by specified address.

```json
{
    "withdrawals": {
        "staker": "terra1..."
    }
}
```

## MigrateMsg

```json
{}
```
