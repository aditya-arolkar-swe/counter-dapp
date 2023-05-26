use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response, StdResult, entry_point, Binary, Deps, to_binary};
use crate::msg::{ExecMsg, InstantiateMsg};

mod contract;
pub mod msg;
mod state;

#[entry_point]
fn instantiate(deps: DepsMut, _env: Env, _info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    contract::instantiate(deps, msg)
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
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: msg::ExecMsg) -> StdResult<Response> {

    match msg {
        ExecMsg::Poke {} => contract::exec::poke(deps, info),
        ExecMsg::Reset { counter } => contract::exec::reset(deps, info, counter),
    };
    Ok(Response::new())
}

#[cfg(test)]
mod test {
    use super::*;

    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use crate::msg::{ExecMsg, QueryMsg, ValueResp};


    fn counting_countract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_countract());

        let contract_addr = app.instantiate_contract(
            contract_id,
            Addr::unchecked("sender"),
            &InstantiateMsg { counter: 20 , minimal_donation: Coin { denom: "COIN".to_string(), amount: Uint128::new(1)} },
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
    fn poke() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_countract());
        let sender = Addr::unchecked("sender");

        let contract_addr = app.instantiate_contract(
            contract_id,
            sender.clone(),
            &InstantiateMsg { counter: 0, minimal_donation: Coin { denom: "COIN".to_string(), amount: "1".parse().unwrap() } },
            &[],
            "Counting contract",
            None
        ).unwrap();

        app.execute_contract(sender, contract_addr.clone(), &ExecMsg::Poke {}, &[])
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 1 })
    }

    #[test]
    fn reset() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_countract());

        let contract_addr = app.instantiate_contract(
            contract_id,
            Addr::unchecked("sender"),
            &InstantiateMsg { counter: 0, minimal_donation: Coin { denom: "COIN".to_string(), amount: "1".parse().unwrap() } },
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
            &InstantiateMsg { counter: 0, minimal_donation: Coin { denom: "COIN".to_string(), amount: "1".parse().unwrap() } },
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
}



