# Vesting

The Vesting Contract contains logic for distributing the token according to the specified vesting schedules for multiple accounts.
Each account can have a different vesting schedules, and the accounts can claim a token at any time after the schedule has passed.

---

## InstantiateMsg

```json
{
    "owner": "terra1...",
    "bro_token": "terra1...",
    "genesis_time": 1642852083
}
```

## ExecuteMsg

### `update_config`

Updates vesting contract config. Can be executed only by owner.
Message params are optional.

```json
{
    "update_config": {
        "owner": "terra1...",
        "genesis_time": 1642852083
    }
}
```

### `register_vesting_accounts`

Registers a list of vesting accounts for future token distribution.

```json
{
    "register_vesting_accounts": {
        "vesting_accounts": [
            {
                "address": "terra1...",
                "schedules": [
                    {
                        "start_time": 1642852083,
                        "end_time": 1642952083,
                        "amount": "10"
                    }
                ]
            }
        ]
    }
}
```

### `claim`

Claims available amount for `sender.address`.

```json
{
    "claim": {}
}
```

## QueryMsg

### `config`

Returns vesting contract config.

```json
{
    "config": {}
}
```

### `vesting_account`

Returns vesting schedules for specified account.

```json
{
    "vesting_account": {
        "address": "terra1..."
    }
}
```

### `vesting_accounts`

Returns paginated vesting schedules using specified filters. 
Query params are optional.

```json
{
    "vesting_accounts": {
        "start_after": "terra1...",
        "limit": 10,
        "order_by": {
            "asc": {}
        }
    }
}
```

### `claimable`

Returns available amount to claim for specified account.

```json
{
    "claimable": {
        "address": "terra1..."
    }
}
```
