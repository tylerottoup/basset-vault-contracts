use commands::repay_logic_on_reply;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128,
    WasmMsg,
};

use crate::{
    commands, queries,
    state::{
        config_set_casset_token, load_borrowing_state, load_config, load_repaying_loan_state,
        store_config, store_state, update_loan_state_part_of_loan_repaid, BorrowingSource,
        RepayingLoanState, State,
    },
};
use crate::{error::ContractError, response::MsgInstantiateContractResponse};
use crate::{state::Config, ContractResult};
use cw20::{Cw20ReceiveMsg, MinterResponse};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use protobuf::Message;
use yield_optimizer::{
    basset_farmer::{
        AnyoneMsg, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, OverseerMsg, QueryMsg,
        YourselfMsg,
    },
    deduct_tax, get_tax_info,
    querier::{query_aterra_state, query_balance, AnchorMarketCw20Msg, AnchorMarketMsg},
};

pub const SUBMSG_ID_INIT_CASSET: u64 = 1;
pub const SUBMSG_ID_REDEEM_STABLE: u64 = 2;
pub const SUBMSG_ID_REPAY_LOAN: u64 = 3;
pub const SUBMSG_ID_BORROWING: u64 = 4;
//withdrawing from Anchor Deposit error
pub const TOO_HIGH_BORROW_DEMAND_ERR_MSG: &str = "borrow demand too high";
//borrowing error
pub const TOO_HIGH_BORROW_AMOUNT_ERR_MSG: &str = "borrow amount too high";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response> {
    let config = Config {
        casset_token: Addr::unchecked(""),
        basset_token: deps.api.addr_validate(&msg.basset_token_addr)?,
        overseer_contract: deps.api.addr_validate(&msg.overseer_addr)?,
        custody_basset_contract: deps.api.addr_validate(&msg.custody_basset_contract)?,
        governance_contract: deps.api.addr_validate(&msg.governance_addr)?,
        anchor_token: deps.api.addr_validate(&msg.anchor_token)?,
        anchor_market_contract: deps.api.addr_validate(&msg.anchor_market_contract)?,
        anchor_ust_swap_contract: deps.api.addr_validate(&msg.anchor_ust_swap_contract)?,
        ust_psi_swap_contract: deps.api.addr_validate(&msg.ust_psi_swap_contract)?,
        aterra_token: deps.api.addr_validate(&msg.aterra_token)?,
        psi_part_in_rewards: msg.psi_part_in_rewards,
        psi_token: deps.api.addr_validate(&msg.psi_token)?,
        basset_farmer_config_contract: deps
            .api
            .addr_validate(&msg.basset_farmer_config_contract)?,
        stable_denom: msg.stable_denom,
    };
    store_config(deps.storage, &config)?;

    let state = State {
        global_reward_index: Decimal256::zero(),
        last_reward_amount: Decimal256::zero(),
    };
    store_state(deps.storage, &state)?;

    Ok(Response {
        messages: vec![],
        submessages: vec![SubMsg {
            msg: WasmMsg::Instantiate {
                admin: None,
                code_id: msg.token_code_id,
                msg: to_binary(&TokenInstantiateMsg {
                    name: "nexus basset token share representation".to_string(),
                    symbol: format!("c{}", msg.collateral_token_symbol),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                })?,
                send: vec![],
                label: "".to_string(),
            }
            .into(),
            gas_limit: None,
            id: SUBMSG_ID_INIT_CASSET,
            reply_on: ReplyOn::Success,
        }],
        attributes: vec![],
        data: None,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> ContractResult<Response> {
    match msg.id {
        SUBMSG_ID_INIT_CASSET => {
            let data = msg.result.unwrap().data.unwrap();
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(data.as_slice())
                .map_err(|_| {
                    StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
                })?;

            let casset_token = res.get_contract_address();
            config_set_casset_token(deps.storage, deps.api.addr_validate(casset_token)?)?;

            Ok(Response {
                messages: vec![],
                submessages: vec![],
                attributes: vec![attr("casset_token_addr", casset_token)],
                data: None,
            })
        }

        SUBMSG_ID_REDEEM_STABLE => match msg.result {
            cosmwasm_std::ContractResult::Err(err_msg) => {
                if err_msg
                    .to_lowercase()
                    .contains(TOO_HIGH_BORROW_DEMAND_ERR_MSG)
                {
                    //we need to repay loan a bit, before redeem stables
                    commands::repay_logic_on_reply(deps, env)
                } else {
                    return Err(StdError::generic_err(format!(
                        "fail to redeem stables, reason: {}",
                        err_msg
                    ))
                    .into());
                }
            }
            cosmwasm_std::ContractResult::Ok(_) => commands::repay_logic_on_reply(deps, env),
        },

        SUBMSG_ID_REPAY_LOAN => {
            let _ = update_loan_state_part_of_loan_repaid(deps.storage)?;
            Ok(Response::default())
        }

        SUBMSG_ID_BORROWING => match msg.result {
            cosmwasm_std::ContractResult::Err(err_msg) => {
                if err_msg
                    .to_lowercase()
                    .contains(TOO_HIGH_BORROW_AMOUNT_ERR_MSG)
                {
                    let borrowing_state = load_borrowing_state(deps.as_ref().storage)?;
                    if borrowing_state.source == BorrowingSource::BassetDeposit {
                        return Err(StdError::generic_err(format!("Borrow limit reached")).into());
                    } else {
                        //TODO: set some flag in database
                        //next - you will have Query that ask about rebalance
                        //and if it needed AND BorrowerAction::Borrow then we check for
                        //that flag - it if is true AND anchor in the same liability state
                        //(same or less deposit amount and same or higher borrow amount)
                        //then return that no action needed
                        todo!()
                    }
                } else {
                    return Err(StdError::generic_err(format!(
                        "fail to borrow stables, reason: {}",
                        err_msg
                    ))
                    .into());
                }
            }
            cosmwasm_std::ContractResult::Ok(_) => {
                todo!()
            }
        },

        unknown => {
            Err(StdError::generic_err(format!("unknown reply message id: {}", unknown)).into())
        }
    }
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
        ExecuteMsg::Yourself { yourself_msg } => match yourself_msg {
            YourselfMsg::AfterBorrow {
                borrowed_amount,
                buffer_size,
            } => {
                todo!()
            }
            YourselfMsg::AfterAterraRedeem { repay_amount } => {
                //TODO: this is how to repay anchor loan
                //
                //CosmosMsg::Wasm(WasmMsg::Execute {
                //    contract_addr: anchor_market_contract,
                //    msg: to_binary(&AnchorMarketMsg::RepayStable {})?,
                //    send: vec![Coin {
                //        denom: stable_denom,
                //        //TODO: is it ok to convert Uint256 to Uint128 - it can throw runtime
                //        //exception
                //        amount: repay_amount.into(),
                //    }],
                //}),
                todo!()
            }
        },
        ExecuteMsg::Anyone { anyone_msg } => match anyone_msg {
            AnyoneMsg::Rebalance => commands::rebalance(deps, env, info),
            AnyoneMsg::Sweep => commands::sweep(deps, env, info),
            AnyoneMsg::SwapAnc => commands::swap_anc(deps, env, info),
            AnyoneMsg::BuyPsiTokens => commands::buy_psi_tokens(deps, env, info),
            AnyoneMsg::DisributeRewards => commands::distribute_rewards(deps, env, info),
            AnyoneMsg::ClaimRewards => commands::claim_rewards(deps, env, info),
        },
        ExecuteMsg::OverseerMsg { overseer_msg } => match overseer_msg {
            OverseerMsg::Deposit { farmer, amount } => {
                commands::deposit_basset(deps, env, info, farmer, amount)
            }
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
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult<Response> {
    Ok(Response::default())
}
