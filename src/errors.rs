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
pub enum CalculateMarketCost {
    NonPositiveQuantity
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProcessMarketOrder {
    OrderAlreadyExists,
    NonPositiveQuantity,
}
