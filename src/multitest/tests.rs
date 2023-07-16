use cosmwasm_std::{Addr, Coin, Empty};
use cw_multi_test::{App, Contract, ContractWrapper};
use crate::{execute, query, instantiate, ContractError};
use crate::multitest::CountingContract;

fn counting_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

const ATOM: &str = "atom";

#[test]
fn query_value() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        Coin::new(10, ATOM)
    ).unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}

#[test]
fn donate() {
    let mut app = App::default();

    let contract_id = app.store_code(counting_contract());
    let sender = Addr::unchecked("sender");

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        "Counting contract",
        Coin::new(10, ATOM)
    ).unwrap();

    contract.donate(&mut app, &sender, &[]).unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 0);
}

#[test]
fn unauthorized_withdraw() {
    let owner = Addr::unchecked("owner");
    let member = Addr::unchecked("member");

    let mut app = App::default();

    let contract_id = app.store_code(counting_contract());

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        "Counting contract",
        Coin::new(10, ATOM)
    ).unwrap();

    let err = contract
        .withdraw(&mut app, &member)
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized { owner: owner.into() })
}