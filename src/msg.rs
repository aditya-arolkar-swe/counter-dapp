use cosmwasm_std::{Coin, Decimal};
use cosmwasm_schema::{cw_serde, QueryResponses};


#[cw_serde]
pub struct Parent {
    pub addr: String,
    pub donating_period: u64,
    pub part: Decimal,
}

#[cw_serde]
pub struct InstantiateMsg {
    #[serde(default)]
    pub counter: u64,
    pub minimal_donation: Coin,
    pub parent: Option<Parent>,
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

#[cw_serde]
pub struct MigrateMsg {
    pub parent: Option<Parent>,
}
