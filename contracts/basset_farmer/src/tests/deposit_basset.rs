use super::sdk::Sdk;
use crate::tests::sdk::{ANCHOR_OVERSEER_CONTRACT, BASSET_TOKEN_ADDR, NASSET_TOKEN_ADDR};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{testing::MOCK_CONTRACT_ADDR, CosmosMsg};
use cosmwasm_std::{to_binary, Uint128, WasmMsg};
use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;

use yield_optimizer::{
    basset_farmer::{AnyoneMsg, ExecuteMsg},
    querier::AnchorOverseerMsg,
};

#[test]
fn deposit_basset() {
    let mut sdk = Sdk::init();

    //first farmer come
    let user_1_address = "addr9999".to_string();
    let deposit_1_amount: Uint256 = 2_000_000_000u128.into();
    {
        // -= USER SEND bAsset tokens to basset_farmer =-
        sdk.set_nasset_supply(Uint256::zero());
        sdk.set_basset_balance(deposit_1_amount);

        let response = sdk
            .user_deposit(&user_1_address, deposit_1_amount.into())
            .unwrap();

        assert_eq!(
            response.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
                    msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                        collaterals: vec![(BASSET_TOKEN_ADDR.to_string(), deposit_1_amount)],
                    })
                    .unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_1_address.clone(),
                        amount: deposit_1_amount.into(), //first depositer have same amount
                    })
                    .unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&ExecuteMsg::Anyone {
                        anyone_msg: AnyoneMsg::Rebalance,
                    })
                    .unwrap(),
                    send: vec![],
                }),
            ]
        );
    }

    //second farmer come
    let user_2_address = "addr6666".to_string();
    let deposit_2_amount: Uint256 = 6_000_000_000u128.into();
    {
        sdk.set_nasset_supply(deposit_1_amount);
        sdk.set_collateral_balance(deposit_1_amount, Uint256::zero());
        sdk.set_basset_balance(deposit_2_amount);
        // -= USER SEND bAsset tokens to basset_farmer =-
        let response = sdk
            .user_deposit(&user_2_address, deposit_2_amount.into())
            .unwrap();
        assert_eq!(
            response.messages,
            vec![
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: ANCHOR_OVERSEER_CONTRACT.to_string(),
                    msg: to_binary(&AnchorOverseerMsg::LockCollateral {
                        collaterals: vec![(BASSET_TOKEN_ADDR.to_string(), deposit_2_amount)],
                    })
                    .unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: NASSET_TOKEN_ADDR.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_2_address.clone(),
                        amount: Uint128(6_000_000_000), //2B * (6B/8B) / (1 - (6B/8B)) = 6B
                    })
                    .unwrap(),
                    send: vec![],
                }),
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                    msg: to_binary(&ExecuteMsg::Anyone {
                        anyone_msg: AnyoneMsg::Rebalance,
                    })
                    .unwrap(),
                    send: vec![],
                }),
            ]
        );
    }
}

#[test]
fn do_not_accept_deposit_if_nluna_supply_is_not_zero_but_bluna_in_custody_is_zero() {
    let mut sdk = Sdk::init();

    sdk.set_nasset_supply(Uint256::one());
    sdk.set_collateral_balance(Uint256::zero(), Uint256::zero());

    //farmer comes
    let user_address = "addr9999".to_string();
    let deposit_amount: Uint256 = 2_000_000_000u128.into();
    // -= USER SEND bAsset tokens to basset_farmer =-
    sdk.set_basset_balance(deposit_amount);

    let response = sdk.user_deposit(&user_address, deposit_amount.into());
    assert!(response.is_err());
}
