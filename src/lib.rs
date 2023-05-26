use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response, StdResult, entry_point, Binary, Deps, to_binary};
use crate::msg::{ExecMsg, InstantiateMsg};

mod contract;
pub mod msg;
mod state;

#[entry_point]
fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;
    use contract::query;

    match msg {
        Value {} => to_binary(&query::value(deps)?),
        Incremented { value } => to_binary(&query::incremented(value))
    }
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: msg::ExecMsg) -> StdResult<Response> {
    match msg {
        ExecMsg::Donate {} => contract::exec::donate(deps, info),
        ExecMsg::Reset { counter } => contract::exec::reset(deps, info, counter),
        ExecMsg::Withdraw {} => contract::exec::withdraw(deps, env, info),
    }?;
    Ok(Response::new())
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{Addr, Coin, coins, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use crate::msg::{ExecMsg, QueryMsg, ValueResp};


    fn counting_countract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    const ATOM: &str = "atom";

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_countract());

        let contract_addr = app.instantiate_contract(
            contract_id,
            Addr::unchecked("sender"),
            &InstantiateMsg { counter: 20 , minimal_donation: Coin::new(10, ATOM) },
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

        let contract_id = app.store_code(counting_countract());
        let sender = Addr::unchecked("sender");

        let contract_addr = app.instantiate_contract(
            contract_id,
            sender.clone(),
            &InstantiateMsg { counter: 0, minimal_donation: Coin::new(10, ATOM) },
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

    #[test]
    fn donate_with_funds() {
        let sender = Addr::unchecked("sender");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, ATOM))
                .unwrap();
        });
        let contract_id = app.store_code(counting_countract());

        let contract_addr = app.instantiate_contract(
            contract_id,
            sender.clone(),
            &InstantiateMsg { counter: 0, minimal_donation: Coin::new(10, ATOM), },
            &[],
            "Counting contract",
            None
        ).unwrap();

         // testing initial funds
        assert_eq!(app.wrap().query_all_balances(&sender).unwrap(), coins(10, ATOM));
        assert_eq!(app.wrap().query_all_balances(&contract_addr).unwrap(), vec![]);


        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, ATOM))
            .unwrap();

        // testing final funds
        assert_eq!(app.wrap().query_all_balances(&sender).unwrap(), vec![]);
        assert_eq!(app.wrap().query_all_balances(&contract_addr).unwrap(), coins(10, ATOM));

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        // testing value incremented
        assert_eq!(resp, ValueResp { value: 1 });

    }

    #[test]
    fn expecting_no_funds() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_countract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    counter: 0,
                    minimal_donation: Coin::new(0, ATOM),
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

        let contract_id = app.store_code(counting_countract());

        let contract_addr = app.instantiate_contract(
            contract_id,
            Addr::unchecked("sender"),
            &InstantiateMsg { counter: 0, minimal_donation: Coin::new(10, ATOM) },
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

        let contract_id = app.store_code(counting_countract());

        let contract_addr = app.instantiate_contract(
            contract_id,
            Addr::unchecked("sender"),
            &InstantiateMsg { counter: 0, minimal_donation: Coin::new(10, ATOM) },
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

    #[test]
    fn withdraw() {
        let owner = Addr::unchecked("owner");
        let sender1 = Addr::unchecked("sender1");
        let sender2 = Addr::unchecked("sender2");

        let mut app = AppBuilder::new().build(|router, _api, storage| {
           router
               .bank
               .init_balance(storage,&sender1, coins(10, ATOM))
               .unwrap();
            router
                .bank
                .init_balance(storage,&sender2, coins(5, ATOM))
                .unwrap();
        });

        let contract_id = app.store_code(counting_countract());

        let contract_addrr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    counter: 0,
                    minimal_donation: Coin::new(10, ATOM),
                },
                &[],
                "Counting contract",
                None,
            ).unwrap();


        app.execute_contract(
            owner.clone(),
            contract_addrr.clone(),
            &ExecMsg::Withdraw {},
            &[],
        ).unwrap();
    }
}



