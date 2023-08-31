use std::fmt::Display;

use rust_decimal::Decimal;

pub type ID = u128;

#[derive(Debug)]
pub struct Order {
    pub id: ID,
    pub side: Side,
    pub price: Decimal,
    pub quantity: Decimal,
    pub priority: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Buy,
    Sell,
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Buy => write!(f, "Buy"),
            Self::Sell => write!(f, "Sell")
        }
    }
}
