use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use rust_decimal::Decimal;

use crate::{
    bookside::{BookSide, MaxPricePriotity, MinPricePriority},
    order::{Order, Side, ID},
};

#[derive(Debug)]
pub struct OrderBook {
    // every active order is in: order_index AND (buy_side XOR sell_side)
    order_index: HashMap<ID, Rc<RefCell<Order>>>,

    buy_side: BookSide<MaxPricePriotity>,
    sell_side: BookSide<MinPricePriority>,

    // increments on each new order added to data structures
    priority: u64,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            order_index: HashMap::new(),

            buy_side: BookSide::new(),
            sell_side: BookSide::new(),

            priority: u64::MIN,
        }
    }

    pub fn process_limit_order(
        &mut self,
        id: ID,
        side: Side,
        price: Decimal,
        mut quantity: Decimal,
    ) -> Result<Vec<OrderMatch>, Error> {
        // check to ensure order does not already exist
        if self.order_index.contains_key(&id) {
            return Err(Error::OrderAlreadyExists);
        }
        // check to ensure positive quantity
        if quantity <= Decimal::ZERO {
            return Err(Error::NonPositiveQuantity);
        }

        // vars
        let mut match_vec = Vec::new();
        let mut new_order_match = OrderMatch::new(id);

        // main matching loop
        while quantity > Decimal::ZERO {
            // get highest priority order on opposite side
            let shared_highest_priority_order = {
                let option_shared_order = match side {
                    Side::Buy => self.sell_side.get_highest_priority(),
                    Side::Sell => self.buy_side.get_highest_priority(),
                };

                match option_shared_order {
                    Some(val) => val,
                    None => break,
                }
            };
            let mut highest_priority_order = shared_highest_priority_order.borrow_mut();

            // check if orders satisfy each other
            let satisfied = match side {
                Side::Buy => price >= highest_priority_order.price,
                Side::Sell => price <= highest_priority_order.price,
            };
            if !satisfied {
                break;
            }

            // create Match for highest_priority_order
            let mut highest_priority_order_match = OrderMatch::new(highest_priority_order.id);

            // find satisfied quantity and update vars
            let satisfied_quantity = quantity.min(highest_priority_order.quantity);

            quantity -= satisfied_quantity;
            highest_priority_order.quantity -= satisfied_quantity;

            new_order_match.quantity += satisfied_quantity;
            highest_priority_order_match.quantity += satisfied_quantity;

            // find cost and update vars
            let buy_side_cost = highest_priority_order.price * satisfied_quantity;
            match side {
                Side::Buy => {
                    new_order_match.cost += buy_side_cost;
                    highest_priority_order_match.cost = -buy_side_cost
                }
                Side::Sell => {
                    new_order_match.cost -= buy_side_cost;
                    highest_priority_order_match.cost = buy_side_cost
                }
            }

            // remove highest_priority_order from orderbook if completely satisfied
            if highest_priority_order.quantity == Decimal::ZERO {
                self.order_index.remove(&highest_priority_order.id);

                match highest_priority_order.side {
                    Side::Buy => {
                        drop(highest_priority_order);
                        self.buy_side.pop_highest_priority();
                    }
                    Side::Sell => {
                        drop(highest_priority_order);
                        self.sell_side.pop_highest_priority();
                    }
                }
            }

            // add to result vec
            match_vec.push(highest_priority_order_match);
        }

        // add to result vec if not empty
        if !new_order_match.quantity.is_zero() {
            match_vec.push(new_order_match);
        }

        // add order to data structures if any remaining quantity
        if !quantity.is_zero() {
            let shared_order = Rc::new(RefCell::new(Order {
                id,
                side,
                price,
                quantity,
                priority: self.get_priority(),
            }));

            self.order_index.insert(id, shared_order.clone());
            match side {
                Side::Buy => self.buy_side.add(shared_order),
                Side::Sell => self.sell_side.add(shared_order),
            }
        }

        Ok(match_vec)
    }

    pub fn cancel_order(&mut self, id: ID) -> Option<Error> {
        match self.order_index.remove(&id) {
            Some(shared_order) => {
                let side;
                {
                    let order = shared_order.borrow();
                    side = order.side;
                }

                match side {
                    Side::Buy => self.buy_side.remove(shared_order),
                    Side::Sell => self.sell_side.remove(shared_order),
                }

                None
            }

            None => Some(Error::OrderNotFound),
        }
    }

    fn get_priority(&mut self) -> u64 {
        let p = self.priority;
        self.priority += 1;
        p
    }
}

impl Display for OrderBook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const PADDING: usize = 18;
        writeln!(
            f,
            "{:->PADDING$}{:->PADDING$}{:->PADDING$}{:->PADDING$}",
            "ID", "SIDE", "PRICE", "QUANTITY"
        )?;

        let mut sell_side: Vec<Rc<RefCell<Order>>> = self.sell_side.iter().map(|(_, shared_order)| shared_order.clone()).collect();
        sell_side.reverse();

        for shared_order in sell_side.iter().chain(self.buy_side.iter().map(|(_, shared_order)| shared_order)) {
            let order = shared_order.borrow();
            writeln!(
                f,
                "{:>PADDING$}{:>PADDING$}{:>PADDING$}{:>PADDING$}",
                order.id,
                order.side.to_string(),
                order.price,
                order.quantity
            )?;
        }

        write!(f, "")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderMatch {
    pub order: ID,
    pub quantity: Decimal,
    pub cost: Decimal,
}

impl OrderMatch {
    fn new(order: ID) -> Self {
        OrderMatch {
            order,
            quantity: Decimal::ZERO,
            cost: Decimal::ZERO,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    OrderAlreadyExists,
    NonPositiveQuantity,
    OrderNotFound,
}
