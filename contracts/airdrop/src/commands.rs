use cosmwasm_std::{DepsMut, Response, MessageInfo, Uint128, CosmosMsg, WasmMsg, to_binary};
use cw20::Cw20ExecuteMsg;
use sha3::Digest;
use std::convert::TryInto;

use crate::{
    error::ContractError,
    state::{
        store_config, load_config, store_latest_stage, load_stage, 
        store_merkle_root, load_merkle_root, store_claimed, read_claimed
    },
};

pub fn update_config(deps: DepsMut, owner: Option<String>) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn register_merkle_root(
    deps: DepsMut,
    merkle_root: String,
) -> Result<Response, ContractError> {
    let mut root_buf: [u8; 32] = [0; 32];
    match hex::decode_to_slice(merkle_root.to_string(), &mut root_buf) {
        Ok(()) => {}
        _ => return Err(ContractError::InvalidHexMerkle {}),
    }

    let latest_stage = load_stage(deps.storage)?;
    let stage = latest_stage + 1;

    store_merkle_root(deps.storage, stage, &merkle_root)?;
    store_latest_stage(deps.storage, stage)?;
    
    Ok(Response::new()
        .add_attributes(vec![
            ("action", "register_merkle_root"),
            ("stage", &stage.to_string()),
            ("merkle_root", &merkle_root),
        ])
    )
}

pub fn claim(
    deps: DepsMut,
    info: MessageInfo,
    stage: u8,
    amount: Uint128,
    proof: Vec<String>,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let merkle_root = load_merkle_root(deps.storage, stage)?;

    let user = info.sender;
    if read_claimed(deps.storage, &user, stage)? {
        return Err(ContractError::AlreadyClaimed {});
    }

    let user_input: String = user.to_string() + &amount.to_string();
    let mut hash: [u8; 32] = sha3::Keccak256::digest(user_input.as_bytes())
        .as_slice()
        .try_into()
        .expect("Wrong length");

    for p in proof {
        let mut proof_buf: [u8; 32] = [0; 32];
        match hex::decode_to_slice(p, &mut proof_buf) {
            Ok(()) => {}
            _ => return Err(ContractError::InvalidHexProof {}),
        }

        hash = if bytes_cmp(hash, proof_buf) == std::cmp::Ordering::Less {
            sha3::Keccak256::digest(&[hash, proof_buf].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        } else {
            sha3::Keccak256::digest(&[proof_buf, hash].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        };
    }

    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root, &mut root_buf).unwrap();
    if root_buf != hash {
        return Err(ContractError::MerkleVerification {});
    }

    store_claimed(deps.storage, &user, stage)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.bro_token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: user.to_string(),
                amount,
            })?,
        })])
        .add_attributes(vec![
            ("action", "claim"),
            ("stage", &stage.to_string()),
            ("address", &user.to_string()),
            ("amount", &amount.to_string()),
        ])
    )
}

fn bytes_cmp(a: [u8; 32], b: [u8; 32]) -> std::cmp::Ordering {
    let mut i = 0;
    while i < 32 {
        match a[i].cmp(&b[i]) {
            std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
            std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
            _ => i += 1,
        }
    }

    std::cmp::Ordering::Equal
}
