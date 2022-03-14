# bBRO Token

This folder implements a modified [CW-20 token standard](https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw20-base), which is the basis for the `$bBro` token.

The differences to a typical CW-20 token is that it's non-transferable. 

A wallet can only hold the token, and redeem it in the brokkr brotocol ecosystem. It can't be send to other wallets or contracts.

---

## InstantiateMsg

```json
{
    "name": "bBRO token",
    "symbol": "bBRO",
    "decimals": 6,
    "initial_balances": [
        {
            "address": "terra1...",
            "amount": "1"
        }
    ],
    "mint": {
        "minter": "terra1..."
    },
    "marketing": {
        "project": "brotocol",
        "description": "some token",
        "marketing": "marketing",
        "logo": {
            "url": "url"
        }
    }
}
```

## ExecuteMsg

### `increase_allowance`

Allows spender to access an additional amount tokens from the owner's (env.sender) account. If expires is Some(), overwrites current allowance expiration with this one.

```json
{
    "increase_allowance": {
        "spender": "terra1...",
        "amount": "1"
    }
}
```

### `decrease_allowance`

Lowers the spender's access of tokens from the owner's (env.sender) account by amount. If expires is Some(), overwrites current allowance expiration with this one.

```json
{
    "decrease_allowance": {
        "spender": "terra1...",
        "amount": "1"
    }
}
```

### `mint`

If authorized, creates amount new tokens and adds to the recipient balance.

```json
{
    "mint": {
        "recipient": "terra1...",
        "amount": "1"
    }
}
```

### `burn_from`

Destroys tokens forever.

```json
{
    "burn_from": {
        "owner": "terra1...",
        "amount": "1",
    }
}
```

### `update_marketing`

If authorized, updates marketing metadata. Setting None/null for any of these will leave it unchanged. Setting Some("") will clear this field on the contract storage.

```json
{
    "update_marketing": {
        "project": "project",
        "description": "desc",
        "marketing": "marketing"
    }
}
```

### `upload_logo`

Upload a new URL, SVG, or PNG for the token.

```json
{
    "upload_logo": {
        "logo": {
            "url": "url"
        }
    }
}
```

## QueryMsg

### `balance`

Returns the current balance of the given address, 0 if unset. Return type: BalanceResponse.

```json
{
    "balance": {
        "address": "terra1..."
    }
}
```

### `token_info`

Returns metadata on the contract - name, decimals, supply, etc. Return type: TokenInfoResponse.

```json
{
    "token_info": {}
}
```

### `minter`

Only with "mintable" extension. Returns who can mint and the hard cap on maximum tokens after minting. Return type: MinterResponse.


```json
{
    "minter": {}
}
```

### `allowance`

Only with "allowance" extension. Returns how much spender can use from owner account, 0 if unset. Return type: AllowanceResponse.

```json
{
    "allowance": {
        "owner": "terra1...",
        "spender": "terra1..."
    }
}
```

### `all_allowances`

Only with "enumerable" extension (and "allowances") Returns all allowances this owner has approved. Supports pagination. Return type: AllAllowancesResponse.

```json
{
    "all_allowances": {
        "owner": "terra1...",
        "start_after": "terra1...",
        "limit": 30
    }
}
```

### `all_accounts`

Only with "enumerable" extension Returns all accounts that have balances. Supports pagination. Return type: AllAccountsResponse.

```json
{
    "all_accounts": {
        "start_after": "terra1...",
        "limit": 30
    }
}
```

### `marketing_info`

Returns more metadata on the contract to display in the client:
description, logo, project url, etc. Return type: MarketingInfoResponse

```json
{
    "marketing_info": {}
}
```

### `download_logo`

Only with "marketing" extension Downloads the embeded logo data (if stored on chain). Errors if no logo data stored for this contract. Return type: DownloadLogoResponse.


```json
{
    "download_logo": {}
}
```

## MigrateMsg

```json
{}
```
