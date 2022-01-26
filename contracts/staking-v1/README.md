# Staking V1

The Staking Contract contains the logic for BRO Token staking and reward distribution.
Also bBRO tokens will be minted as a reward for staking BRO.

---

## InstantiateMsg

```json
{
    "bro_token": "terra1...",
    "rewards_pool_contract": "terra1...",
    "bbro_minter_contract": "terra1...",
    "epoch_manager_contract": "terra1...",
    "unbond_period_blocks": 100
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

### `receive:bond`

Deposits specified amount of tokens to get reward shares.

```json
{
    "bond": {}
}
```

### `unbond`

Unbond staked amount of tokens. Tokens will be claimable only after passing unbonding period.

```json
{
    "unbond": {
        "amount": "100"
    }
}
```

### `withdraw`

Withdraw amount of tokens which have already passed unbonding period.

```json
{
    "withdraw": {}
}
```

### `claim_rewards`

Claim availalble reward amount.

```json
{
    "claim_rewards": {}
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

Returns available amount for staker to withdraw by specified address.

```json
{
    "withdrawals": {
        "staker": "terra1..."
    }
}
```
