use std::fmt::Display;

use rust_decimal::Decimal;

#[derive(Debug)]
pub struct Order<ID> {
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

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Buy => write!(f, "Buy"),
            Self::Sell => write!(f, "Sell"),
        }
    }
}
