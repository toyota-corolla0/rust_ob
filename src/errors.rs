#[derive(Debug, PartialEq, Clone)]
pub enum ProcessLimitOrder {
    OrderAlreadyExists,
    NonPositiveQuantity,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CancelOrder {
    OrderNotFound,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FindMarketCost {
    NonPositiveQuantity
}

pub enum ProcessMarketOrder {
    OrderAlreadyExists,
    NonPositiveQuantity,
}
