use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Decimal,
    Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg,
    Uint128, WasmMsg,
};

use std::collections::VecDeque;

use crate::{
    commands, queries,
    state::{load_config, save_state, State, PRICES_COUNT},
};
use crate::{error::ContractError, state::load_state};
use crate::{
    state::{Config, CONFIG},
    ContractResult,
};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use yield_optimizer::{
    basset_farmer_config::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    querier::{
        get_basset_in_custody, query_price, query_supply, query_token_balance, PriceResponse,
        TimeConstraints,
    },
};

//fn calc_price_variance(prices: &VecDeque<Decimal256>, elems_count: u64) -> Decimal256 {
//    let prices_sum: Decimal256 = prices
//        .iter()
//        .fold(Decimal256::zero(), |result, p| result + *p);
//    // let avg_price = Decimal256::from_ratio(prices_sum, prices.len());
//    let decimal_elems_count = Decimal256::from_uint256(Uint256::from(elems_count));
//    let mean_price = prices_sum / decimal_elems_count;

//    let diviations_sum: Decimal256 = prices
//        .iter()
//        .map(|p| safe_sub(*p, mean_price) * safe_sub(*p, mean_price))
//        .fold(Decimal256::zero(), |result, p| result + p);
//    println!("diviations_sum: {}", diviations_sum);

//    let variance_square: Decimal256 = (diviations_sum / decimal_elems_count);

//    //TODO: variance = sqrt(variance_square);
//    //TODO: result = variance / mean_price * 100;
//    return variance_square;
//}

//fn safe_sub(x: Decimal256, y: Decimal256) -> Decimal256 {
//    if x > y {
//        x - y
//    } else if y > x {
//        y - x
//    } else {
//        Decimal256::zero()
//    }
//}

/// Executor: governance
pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    borrow_ration_aim: Option<Decimal>,
    borrow_ration_upper_gap: Option<Decimal>,
    borrow_ration_bottom_gap: Option<Decimal>,
    oracle_addr: Option<String>,
    basset_token_addr: Option<String>,
    stable_denom: Option<String>,
) -> ContractResult<Response> {
    let config = load_config(deps.storage)?;
    if info.sender != config.governance_contract_addr {
        return Err(ContractError::Unauthorized {});
    }

    todo!()
}

// #[cfg(test)]
// mod test {
//     use cosmwasm_bignumber::{Decimal256, Uint256};
//     use std::collections::VecDeque;

//     use super::calc_price_variance;

//     #[test]
//     fn calc_variance_1() {
//         let prices: VecDeque<Decimal256> = VecDeque::from(vec![
//             Decimal256::from_uint256(Uint256::from(14u64)),
//             Decimal256::from_uint256(Uint256::from(14u64)),
//             Decimal256::from_uint256(Uint256::from(0u64)),
//             Decimal256::from_uint256(Uint256::from(0u64)),
//         ]);

//         let variance = calc_price_variance(&prices, 4);
//         assert_eq!(Decimal256::from_uint256(Uint256::from(7u64)), variance);
//     }
// }
