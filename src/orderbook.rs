use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use rust_decimal::Decimal;

use crate::{
    bookside::{BookSide, MaxPricePriotity, MinPricePriority},
    errors,
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
    /// Create new initialized OrderBook
    pub fn new() -> Self {
        OrderBook {
            order_index: HashMap::new(),

            buy_side: BookSide::new(),
            sell_side: BookSide::new(),

            priority: u64::MIN,
        }
    }

    /// Process new limit order
    /// ```
    /// use rust_ob::{
    ///     OrderBook,
    ///     Side,
    ///     OrderMatch,
    ///     errors,
    /// };
    /// use rust_decimal::Decimal;
    ///
    /// let mut ob = OrderBook::new();
    ///
    /// let res1 = ob.process_limit_order(1, Side::Sell, Decimal::from(4), Decimal::from(4)).unwrap();
    /// assert_eq!(res1.len(), 0);
    ///
    /// let res2 = ob.process_limit_order(2, Side::Sell, Decimal::from(3), Decimal::from(2)).unwrap();
    /// assert_eq!(res2.len(), 0);
    ///
    /// let res3 = ob.process_limit_order(3, Side::Buy, Decimal::from(8), Decimal::from(3)).unwrap();
    /// assert_eq!(
    ///     res3,
    ///     vec![
    ///         OrderMatch {
    ///             order: 2,
    ///             quantity: Decimal::from(2),
    ///             cost: Decimal::from(-6)
    ///         },
    ///         OrderMatch {
    ///             order: 1,
    ///             quantity: Decimal::from(1),
    ///             cost: Decimal::from(-4)
    ///         },
    ///         OrderMatch {
    ///             order: 3,
    ///             quantity: Decimal::from(3),
    ///             cost: Decimal::from(10)
    ///         }
    ///     ]
    /// );
    ///
    ///
    /// // all costs sum to zero
    /// assert_eq!(res3.iter().map(|val| val.cost).sum::<Decimal>(), Decimal::ZERO);
    ///
    /// // quantity on sell orders == quantity on buy orders
    /// // last OrderMatch of Vec (if not empty) is always the order just placed
    /// assert_eq!(res3.iter().map(|val| val.quantity).sum::<Decimal>(), res3.last().unwrap().quantity * Decimal::from(2));
    ///
    /// // possible errors
    /// assert_eq!(ob.process_limit_order(4, Side::Buy, Decimal::from(10), Decimal::from(0)).unwrap_err(), errors::ProcessLimitOrder::NonPositiveQuantity);
    /// assert_eq!(ob.process_limit_order(1, Side::Buy, Decimal::from(10), Decimal::from(25)).unwrap_err(), errors::ProcessLimitOrder::OrderAlreadyExists);
    ///
    ///
    /// ```
    pub fn process_limit_order(
        &mut self,
        id: ID,
        side: Side,
        price: Decimal,
        mut quantity: Decimal,
    ) -> Result<Vec<OrderMatch>, errors::ProcessLimitOrder> {
        // check to ensure order does not already exist
        if self.order_index.contains_key(&id) {
            return Err(errors::ProcessLimitOrder::OrderAlreadyExists);
        }
        // check to ensure positive quantity
        if quantity <= Decimal::ZERO {
            return Err(errors::ProcessLimitOrder::NonPositiveQuantity);
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

            quantity = quantity
                .checked_sub(satisfied_quantity)
                .unwrap_or_else(|| panic!("OrderBook: subtraction overflow"));
            highest_priority_order.quantity = highest_priority_order
                .quantity
                .checked_sub(satisfied_quantity)
                .unwrap_or_else(|| panic!("OrderBook: subtraction overflow"));

            new_order_match.quantity = new_order_match
                .quantity
                .checked_add(satisfied_quantity)
                .unwrap_or_else(|| panic!("OrderBook: addition overflow"));
            highest_priority_order_match.quantity = highest_priority_order_match
                .quantity
                .checked_add(satisfied_quantity)
                .unwrap_or_else(|| panic!("OrderBook: addition overflow"));

            // find cost and update vars
            let buy_side_cost = highest_priority_order
                .price
                .checked_mul(satisfied_quantity)
                .unwrap_or_else(|| panic!("OrderBook: multiplication overflow"));
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

    /// Cancels order with id
    /// ```
    /// use rust_ob::{
    ///     OrderBook,
    ///     Side,
    ///     errors,
    /// };
    /// use rust_decimal::Decimal;
    ///
    /// let mut ob = OrderBook::new();
    /// let _ = ob.process_limit_order(884213, Side::Sell, Decimal::from(5), Decimal::from(5));
    ///
    /// assert_eq!(ob.cancel_order(884213), Ok(()));
    ///
    /// // possible errors
    /// assert_eq!(ob.cancel_order(884213), Err(errors::CancelOrder::OrderNotFound));
    /// ```
    pub fn cancel_order(&mut self, id: ID) -> Result<(), errors::CancelOrder> {
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

                Ok(())
            }

            None => Err(errors::CancelOrder::OrderNotFound),
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

        let sell_side: Vec<Rc<RefCell<Order>>> = self
            .sell_side
            .iter()
            .map(|(_, shared_order)| shared_order.clone())
            .collect();

        for shared_order in sell_side
            .iter()
            .rev()
            .chain(self.buy_side.iter().map(|(_, shared_order)| shared_order))
        {
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
