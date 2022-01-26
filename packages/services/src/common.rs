use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Order;

/// ## OrderBy
/// This enum describes the type of sort
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderBy {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

// We suppress this clippy warning because Order in cosmwasm doesn't implement Debug and
// PartialEq for usage in QueryMsg, we need to use our own OrderBy and
// convert it finally to cosmwasm's Order
#[allow(clippy::from_over_into)]
impl Into<Order> for OrderBy {
    fn into(self) -> Order {
        if self == OrderBy::Asc {
            Order::Ascending
        } else {
            Order::Descending
        }
    }
}
