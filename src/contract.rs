use cosmwasm_std::{DepsMut, Response, StdResult};
use crate::InstantiateMsg;
use crate::state::{COUNTER, MINIMAL_DONATION};

pub fn instantiate(deps: DepsMut, msg: InstantiateMsg) -> StdResult<Response> {
    COUNTER.save(deps.storage, &msg.counter)?;
    MINIMAL_DONATION.save(deps.storage, &msg.minimal_donation)?;
    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};
    use crate::msg::ValueResp;
    use crate::state::COUNTER;

    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = COUNTER.load(deps.storage)?;
        Ok(ValueResp { value })
    }

    pub fn incremented(value: u64) -> ValueResp {
        ValueResp { value: value + 1 }
    }
}

pub mod exec {
    use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};
    use crate::state::COUNTER;

    pub fn poke(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let value = COUNTER.load(deps.storage)? + 1;
        COUNTER.save(deps.storage, &value)?;

        let resp = Response::new()
            .add_attribute("action", "poke")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("value", &value.to_string());

        Ok(resp)
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, counter: u64) -> StdResult<Response> {
        COUNTER.save(deps.storage, &counter)?;

        let resp = Response::new()
            .add_attribute("action", "reset")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", counter.to_string());

        Ok(resp)
    }
}