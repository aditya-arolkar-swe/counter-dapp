use cosmwasm_std::Coin;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    #[serde(default)]
    pub counter: u64,
    pub minimal_donation: Coin,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ValueResp)]
    Value {},
    #[returns(ValueResp)]
    Incremented { value: u64 },

}

#[cw_serde]
pub enum ExecMsg {
    Donate {},
    Withdraw {},
    Reset {
        #[serde(default)]
        counter: u64,
    }
}

#[cw_serde]
pub struct ValueResp {
    pub value: u64,
}
