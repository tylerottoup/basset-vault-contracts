use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};

use crate::{
    commands, queries,
    state::{store_config, store_state, Config, State},
    ContractResult,
};
use yield_optimizer::nasset_staker::{AnyoneMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config {
        nasset_token: deps.api.addr_validate(&msg.nasset_token)?,
        psi_token: deps.api.addr_validate(&msg.psi_token)?,
        governance_addr: deps.api.addr_validate(&msg.governance_contract)?,
    };
    store_config(deps.storage, &config)?;

    let state = State {
        global_reward_index: Decimal256::zero(),
        last_reward_amount: Uint256::zero(),
        total_staked_amount: Uint256::zero(),
    };
    store_state(deps.storage, &state)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => commands::receive_cw20(deps, env, info, msg),
        ExecuteMsg::Anyone { anyone_msg } => match anyone_msg {
            AnyoneMsg::UpdateIndex => commands::update_global_index(deps, env),

            AnyoneMsg::ClaimRewards { to } => commands::claim_rewards(deps, env, info.sender, to),

            AnyoneMsg::Unstake { amount, to } => {
                commands::unstake_nasset(deps, env, info.sender, amount, to)
            }

            AnyoneMsg::ClaimRemainder => commands::claim_remainder(deps, env),
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config => to_binary(&queries::query_config(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> ContractResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
