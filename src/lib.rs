#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, Binary, Deps, to_binary, Empty};
use crate::error::ContractError;
use crate::msg::{ExecMsg, InstantiateMsg, MigrateMsg};
mod contract;
pub mod msg;
mod state;
pub mod error;
#[cfg(any(test, feature = "tests"))]
pub mod multitest;

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;
    use contract::query;

    match msg {
        Value {} => to_binary(&query::value(deps)?),
        Incremented { value } => to_binary(&query::incremented(value))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: msg::ExecMsg) -> Result<Response, ContractError> {
    match msg {
        ExecMsg::Donate {} => contract::exec::donate(deps, env, info).map_err(ContractError::Std),
        ExecMsg::Reset { counter } => contract::exec::reset(deps, info, counter).map_err(ContractError::Std),
        ExecMsg::Withdraw {} => contract::exec::withdraw(deps, env, info),
    }?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    contract::migrate(deps, msg)
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(unused_imports)]
    use cosmwasm_std::{Addr, Coin, coin, coins, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use crate::msg::{ExecMsg, QueryMsg, ValueResp};


    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    const ATOM: &str = "atom";

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app.instantiate_contract(
            contract_id,
            Addr::unchecked("sender"),
            &InstantiateMsg { counter: 20 , minimal_donation: Coin::new(10, ATOM), parent: None },
            &[],
            "Counting contract",
            None
        ).unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 20 })
    }

    #[test]
    fn donate() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let sender = Addr::unchecked("sender");

        let contract_addr = app.instantiate_contract(
            contract_id,
            sender.clone(),
            &InstantiateMsg { counter: 0, minimal_donation: Coin::new(10, ATOM), parent: None },
            &[],
            "Counting contract",
            None
        ).unwrap();

        app.execute_contract(sender, contract_addr.clone(), &ExecMsg::Donate {}, &[])
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 0 })
    }

    // #[test]
    // fn donate_with_funds() {
    //     let sender = Addr::unchecked("sender");
    //
    //     let mut app = App::new(|router, _api, storage| {
    //         router
    //             .bank
    //             .init_balance(storage, &sender, coins(10, ATOM))
    //             .unwrap();
    //     });
    //     let contract_id = app.store_code(counting_contract());
    //
    //     let contract_addr = app.instantiate_contract(
    //         contract_id,
    //         sender.clone(),
    //         &InstantiateMsg { counter: 0, minimal_donation: Coin::new(10, ATOM), },
    //         &[],
    //         "Counting contract",
    //         None
    //     ).unwrap();
    //
    //      // testing initial funds
    //     assert_eq!(app.wrap().query_all_balances(&sender).unwrap(), coins(10, ATOM));
    //     assert_eq!(app.wrap().query_all_balances(&contract_addr).unwrap(), vec![]);
    //
    //
    //     app.execute_contract(
    //         sender.clone(),
    //         contract_addr.clone(),
    //         &ExecMsg::Donate {},
    //         &coins(10, ATOM))
    //         .unwrap();
    //
    //     // testing final funds
    //     assert_eq!(app.wrap().query_all_balances(&sender).unwrap(), vec![]);
    //     assert_eq!(app.wrap().query_all_balances(&contract_addr).unwrap(), coins(10, ATOM));
    //
    //     let resp: ValueResp = app
    //         .wrap()
    //         .query_wasm_smart(contract_addr, &QueryMsg::Value {})
    //         .unwrap();
    //
    //     // testing value incremented
    //     assert_eq!(resp, ValueResp { value: 1 });
    //
    // }

    #[test]
    fn expecting_no_funds() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    counter: 0,
                    minimal_donation: Coin::new(0, ATOM),
                    parent: None
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &[],
        )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1 });
    }

    #[test]
    fn reset() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app.instantiate_contract(
            contract_id,
            Addr::unchecked("sender"),
            &InstantiateMsg { counter: 0, minimal_donation: Coin::new(10, ATOM), parent: None },
            &[],
            "Counting contract",
            None
        ).unwrap();

        app.execute_contract(Addr::unchecked("sender"), contract_addr.clone(), &ExecMsg::Reset { counter : 10}, &[])
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 10 })
    }

    #[test]
    fn query_incremented() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app.instantiate_contract(
            contract_id,
            Addr::unchecked("sender"),
            &InstantiateMsg { counter: 0, minimal_donation: Coin::new(10, ATOM), parent: None },
            &[],
            "Counting contract",
            None
        ).unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Incremented { value: 1 })
            .unwrap();

        assert_eq!(resp, ValueResp { value: 2 })
    }

    // #[test]
    // fn withdraw() {
    //     let owner = Addr::unchecked("owner");
    //     let sender = Addr::unchecked("sender");
    //
    //     let mut app = App::new(|router, _api, storage| {
    //         router
    //             .bank
    //             .init_balance(storage, &sender, coins(10, "atom"))
    //             .unwrap();
    //     });
    //
    //     let contract_id = app.store_code(counting_contract());
    //
    //     let contract_addr = app
    //         .instantiate_contract(
    //             contract_id,
    //             owner.clone(),
    //             &InstantiateMsg {
    //                 counter: 0,
    //                 minimal_donation: coin(10, "atom"),
    //             },
    //             &[],
    //             "Counting contract",
    //             None,
    //         )
    //         .unwrap();
    //
    //     app.execute_contract(
    //         sender.clone(),
    //         contract_addr.clone(),
    //         &ExecMsg::Donate {},
    //         &coins(10, "atom"),
    //     )
    //         .unwrap();
    //
    //     app.execute_contract(
    //         owner.clone(),
    //         contract_addr.clone(),
    //         &ExecMsg::Withdraw {},
    //         &[],
    //     )
    //         .unwrap();
    //
    //     // the owner has 10 atom after the sender donated
    //     assert_eq!(app.wrap().query_all_balances(owner).unwrap(), coins(10, "atom"));
    //
    //     // the sender has no coins after donating the 10 atom he was instantiated with
    //     assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
    //
    //     // the contract address has no stored coins in the process
    //     assert_eq!(app.wrap().query_all_balances(contract_addr).unwrap(), vec![]);
    // }

    #[test]
    fn unauthorized_withdraw() {
        let owner = Addr::unchecked("owner");
        let member = Addr::unchecked("member");

        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    counter: 0,
                    minimal_donation: coin(10, "atom"),
                    parent: None
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(member,contract_addr.clone(),&ExecMsg::Withdraw {},&[])
            .unwrap_err();

        assert_eq!(ContractError::Unauthorized { owner: owner.into()}, err.downcast().unwrap())
    }
}



