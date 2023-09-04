#[derive(Debug, PartialEq, Clone)]
pub enum ProcessLimitOrder {
    OrderAlreadyExists,
    NonPositiveQuantity,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CancelOrder {
    OrderNotFound,
}