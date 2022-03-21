use cosmwasm_std::{
    CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage,
};
use cw20::Expiration;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// ## Description
/// Stores ownership proposal struct of type [`OwnershipProposal`] at the given key
static OWNERSHIP_PROPOSAL: Item<OwnershipProposal> = Item::new("ownership_proposal");

/// ## Description
/// Specific type for updating owner fn
type UpdateOwnerFn = fn(&mut dyn Storage, CanonicalAddr) -> StdResult<()>;

/// ## Description
/// This structure describes the basic settings for creating a request for a change of ownership
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub(crate) struct OwnershipProposal {
    // address that was proposed as a new owner
    pub proposed_owner: CanonicalAddr,
    // proposal expiration time in blocks
    pub expires_at: Expiration,
}

/// ## Description
/// Creates a new request to change ownership.
/// Returns an [`OwnershipProposalError`] on failure or returns the [`Response`] with the specified attributes if the operation was successful
/// ## Executor
/// Only owner can execute it
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **info** is an object of type [`MessageInfo`]
///
/// * **env** is an object of type [`Env`]
///
/// * **owner** is an object of type [`CanonicalAddr`]
///
/// * **new_owner** is an object of type [`String`]
///
/// * **expires_in_blocks** is a field of type [`u64`]
pub fn propose_new_owner(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: CanonicalAddr,
    new_owner: String,
    expires_in_blocks: u64,
) -> Result<Response, OwnershipProposalError> {
    if deps.api.addr_canonicalize(&info.sender.to_string())? != owner {
        return Err(OwnershipProposalError::Unauthorized {});
    }

    let new_owner_raw = deps.api.addr_canonicalize(&new_owner)?;
    if new_owner_raw == owner {
        return Err(OwnershipProposalError::SameAddressForProposal {});
    }

    OWNERSHIP_PROPOSAL.save(
        deps.storage,
        &OwnershipProposal {
            proposed_owner: new_owner_raw,
            expires_at: Expiration::AtHeight(env.block.height + expires_in_blocks),
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        ("action", "propose_new_owner"),
        ("proposed_owner", &new_owner),
    ]))
}

/// ## Description
/// Removes a request to change ownership.
/// Returns an [`OwnershipProposalError`] on failure or returns the [`Response`] with the specified attributes if the operation was successful
/// ## Executor
/// Only owner can execute it
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **info** is an object of type [`MessageInfo`]
///
/// * **owner** is an object of type [`CanonicalAddr`]
pub fn drop_ownership_proposal(
    deps: DepsMut,
    info: MessageInfo,
    owner: CanonicalAddr,
) -> Result<Response, OwnershipProposalError> {
    if deps.api.addr_canonicalize(&info.sender.to_string())? != owner {
        return Err(OwnershipProposalError::Unauthorized {});
    }

    OWNERSHIP_PROPOSAL.remove(deps.storage);

    Ok(Response::new().add_attributes(vec![("action", "drop_ownership_proposal")]))
}

/// ## Description
/// Approves new owner proposal.
/// Returns an [`OwnershipProposalError`] on failure or returns the [`Response`] with the specified attributes if the operation was successful
/// ## Executor
/// Only owner can execute it
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
///
/// * **update_owner_fn** is an object of type [`UpdateOwnerFn`]
pub fn claim_ownership(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    update_owner_fn: UpdateOwnerFn,
) -> Result<Response, OwnershipProposalError> {
    let proposal: OwnershipProposal = OWNERSHIP_PROPOSAL
        .load(deps.storage)
        .map_err(|_| OwnershipProposalError::NotFound {})?;

    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    if sender_raw != proposal.proposed_owner {
        return Err(OwnershipProposalError::Unauthorized {});
    }

    if proposal.expires_at.is_expired(&env.block) {
        return Err(OwnershipProposalError::ProposalHasExpired {});
    }

    OWNERSHIP_PROPOSAL.remove(deps.storage);

    update_owner_fn(deps.storage, sender_raw)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "claim_ownership"),
        ("new_owner", &info.sender.to_string()),
    ]))
}

/// ## OwnershipProposalResponse
/// This structure describes the fields for ownership proposal response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnershipProposalResponse {
    // address that was proposed as a new owner
    pub proposed_owner: String,
    // proposal expiration time in blocks
    pub expires_at: Expiration,
}

/// ## Description
/// Returns information about created ownership proposal in the [`OwnershipProposalResponse`] object
/// otherwise returns not-found error
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_ownership_proposal(deps: Deps) -> StdResult<OwnershipProposalResponse> {
    let proposal: OwnershipProposal = OWNERSHIP_PROPOSAL
        .load(deps.storage)
        .map_err(|_| StdError::generic_err("Ownership proposal not found"))?;

    let resp = OwnershipProposalResponse {
        proposed_owner: deps
            .api
            .addr_humanize(&proposal.proposed_owner)?
            .to_string(),
        expires_at: proposal.expires_at,
    };

    Ok(resp)
}

/// ## Description
/// This enum describes ownership proposal errors
#[derive(Error, Debug, PartialEq)]
pub enum OwnershipProposalError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("The address of the new owner cannot be the same as the address of the current")]
    SameAddressForProposal {},

    #[error("Ownership proposal not found")]
    NotFound {},

    #[error("Ownership proposal expired")]
    ProposalHasExpired {},
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Api, Attribute};

    #[test]
    fn update_owner() {
        let mut deps = mock_dependencies(&[]);
        let mut env = mock_env();
        env.block.height = 13000;

        let current_owner = deps.api.addr_canonicalize("owner0000").unwrap();

        // create proposal
        // error: Unauthorized
        let info = mock_info("addr0000", &[]);
        let res = propose_new_owner(
            deps.as_mut(),
            env.clone(),
            info,
            current_owner.clone(),
            "owner0001".to_string(),
            100,
        );

        assert_eq!(res.unwrap_err(), OwnershipProposalError::Unauthorized {});

        // error: propose current owner
        let info = mock_info("owner0000", &[]);
        let res = propose_new_owner(
            deps.as_mut(),
            env.clone(),
            info,
            current_owner.clone(),
            "owner0000".to_string(),
            100,
        );

        assert_eq!(
            res.unwrap_err(),
            OwnershipProposalError::SameAddressForProposal {}
        );

        // proper proposal
        let info = mock_info("owner0000", &[]);
        let res = propose_new_owner(
            deps.as_mut(),
            env.clone(),
            info,
            current_owner.clone(),
            "owner0001".to_string(),
            100,
        )
        .unwrap();

        assert_eq!(
            res.attributes[0],
            Attribute::new("action", "propose_new_owner")
        );
        assert_eq!(
            res.attributes[1],
            Attribute::new("proposed_owner", "owner0001")
        );

        assert_eq!(
            query_ownership_proposal(deps.as_ref()).unwrap(),
            OwnershipProposalResponse {
                proposed_owner: "owner0001".to_string(),
                expires_at: Expiration::AtHeight(13000 + 100),
            }
        );

        // drop proposal
        // error: Unauthorized
        let info = mock_info("owner0001", &[]);
        let res = drop_ownership_proposal(deps.as_mut(), info, current_owner.clone());

        assert_eq!(res.unwrap_err(), OwnershipProposalError::Unauthorized {});

        // proper proposal drop
        let info = mock_info("owner0000", &[]);
        let res = drop_ownership_proposal(deps.as_mut(), info, current_owner.clone()).unwrap();

        assert_eq!(
            res.attributes[0],
            Attribute::new("action", "drop_ownership_proposal")
        );

        // must be removed from storage
        assert_eq!(
            query_ownership_proposal(deps.as_ref()).unwrap_err(),
            StdError::generic_err("Ownership proposal not found"),
        );

        let mock_update_fn: UpdateOwnerFn = |_, _| Ok(());

        // error: claim ownership with no proposal
        let info = mock_info("owner0001", &[]);
        let res = claim_ownership(deps.as_mut(), env.clone(), info, mock_update_fn);

        assert_eq!(res.unwrap_err(), OwnershipProposalError::NotFound {});

        // restore proposal
        let info = mock_info("owner0000", &[]);
        let _res = propose_new_owner(
            deps.as_mut(),
            env.clone(),
            info,
            current_owner.clone(),
            "owner0001".to_string(),
            100,
        )
        .unwrap();

        // error: sender is not a proposed owner
        let info = mock_info("owner0002", &[]);
        let res = claim_ownership(deps.as_mut(), env.clone(), info, mock_update_fn);

        assert_eq!(res.unwrap_err(), OwnershipProposalError::Unauthorized {});

        // error: proposal has expired
        env.block.height = 13101;

        let info = mock_info("owner0001", &[]);
        let res = claim_ownership(deps.as_mut(), env.clone(), info, mock_update_fn);

        assert_eq!(
            res.unwrap_err(),
            OwnershipProposalError::ProposalHasExpired {}
        );

        // proper ownership claim
        env.block.height = 13099;

        let info = mock_info("owner0001", &[]);
        let res = claim_ownership(deps.as_mut(), env.clone(), info, mock_update_fn).unwrap();

        assert_eq!(
            res.attributes[0],
            Attribute::new("action", "claim_ownership")
        );
        assert_eq!(res.attributes[1], Attribute::new("new_owner", "owner0001"));

        // proposal removed
        assert_eq!(
            query_ownership_proposal(deps.as_ref()).unwrap_err(),
            StdError::generic_err("Ownership proposal not found"),
        );
    }
}
