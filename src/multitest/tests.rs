use cosmwasm_std::{Addr, Coin, coins, Empty};
use crate::{execute, query, instantiate, ContractError};
use crate::multitest::CountingContract;
use counting_contract_0_1_0::multitest::CountingContract as CountingContract0_1_0;
use cw_multi_test::{App, Contract, ContractWrapper};
use crate::state::{STATE, State};

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
        None,
        "Counting contract",
        Coin::new(10, ATOM)
    ).unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 1);
}

#[test]
fn donate() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::default();
    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Counting contract",
        Coin::new(10, ATOM)
    ).unwrap();

    contract.donate(&mut app, &sender, &[]).unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 1);
    // assert_eq!(resp.value, 0);
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
        None,
        "Counting contract",
        Coin::new(10, ATOM)
    ).unwrap();

    let err = contract
        .withdraw(&mut app, &member)
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized { owner: owner.into() })
}

#[test]
fn migration() {
    let owner = Addr::unchecked("owner");
    let admin = Addr::unchecked("admin");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(
        |router, _api, storage| {
            router.bank.init_balance(storage, &sender, coins(10, ATOM)).unwrap();
        }
    );

    let old_code_id = CountingContract0_1_0::store_code(&mut app);
    let new_code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract0_1_0::instantiate(
        &mut app, old_code_id, &owner, Some(&admin), "Counting contract", Coin::new(10, ATOM)
    ).unwrap();

    contract.donate(&mut app, &sender, &coins(10, ATOM)).unwrap();

    let contract = CountingContract::migrate(&mut app, &admin, contract.addr(), new_code_id).unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp.value, 0);

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(state, State { counter: 1, minimal_donation: Coin::new(10, ATOM) })

}
