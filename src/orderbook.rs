use std::{cell::RefCell, collections::HashMap, fmt::Display, hash::Hash, rc::Rc};

use rust_decimal::Decimal;

use crate::{
    bookside::{BookSide, MaxPricePriority, MinPricePriority},
    errors,
    order::{Order, Side},
};

#[derive(Debug)]
pub struct OrderBook<OrderID>
where
    OrderID: Copy + PartialEq + Eq + Hash,
{
    // every active order is in: order_index AND (buy_side XOR sell_side)
    order_index: HashMap<OrderID, Rc<RefCell<Order<OrderID>>>>,

    buy_side: BookSide<MaxPricePriority, OrderID>,
    sell_side: BookSide<MinPricePriority, OrderID>,

    // increments on each new order added to data structures. Used for order time priority.
    priority: u64,
}

impl<OrderID> OrderBook<OrderID>
where
    OrderID: Copy + PartialEq + Eq + Hash,
{
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
    /// assert_eq!(res3.iter().take(res3.len()-1).map(|val| val.quantity).sum::<Decimal>(), res3.last().unwrap().quantity);
    ///
    /// // possible errors
    /// assert_eq!(ob.process_limit_order(4, Side::Buy, Decimal::from(10), Decimal::from(0)).unwrap_err(), errors::ProcessLimitOrder::NonPositiveQuantity);
    /// assert_eq!(ob.process_limit_order(1, Side::Buy, Decimal::from(10), Decimal::from(25)).unwrap_err(), errors::ProcessLimitOrder::OrderAlreadyExists);
    /// ```
    pub fn process_limit_order(
        &mut self,
        id: OrderID,
        side: Side,
        price: Decimal,
        mut quantity: Decimal,
    ) -> Result<Vec<OrderMatch<OrderID>>, errors::ProcessLimitOrder> {
        // check to ensure order does not already exist
        if self.order_index.contains_key(&id) {
            return Err(errors::ProcessLimitOrder::OrderAlreadyExists);
        }
        // check to ensure positive quantity
        if quantity <= Decimal::ZERO {
            return Err(errors::ProcessLimitOrder::NonPositiveQuantity);
        }

        // vars
        let mut order_match_vec = Vec::new();
        let mut new_order_order_match = OrderMatch::new(id);

        // main matching loop
        while quantity > Decimal::ZERO {
            // get highest priority order on opposite side
            let Some(shared_highest_priority_order) = (match side {
                Side::Buy => self.sell_side.get_highest_priority(),
                Side::Sell => self.buy_side.get_highest_priority(),
            }) else {
                break;
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

            // create order match for highest_priority_order
            let mut highest_priority_order_order_match = OrderMatch::new(highest_priority_order.id);

            // find satisfied quantity and update vars
            let satisfied_quantity = quantity.min(highest_priority_order.quantity);

            quantity -= satisfied_quantity;
            highest_priority_order.quantity -= satisfied_quantity;

            new_order_order_match.quantity += satisfied_quantity;
            highest_priority_order_order_match.quantity += satisfied_quantity;

            // find cost and update vars
            let buy_side_cost = highest_priority_order.price * satisfied_quantity;
            match side {
                Side::Buy => {
                    new_order_order_match.cost += buy_side_cost;
                    highest_priority_order_order_match.cost = -buy_side_cost
                }
                Side::Sell => {
                    new_order_order_match.cost -= buy_side_cost;
                    highest_priority_order_order_match.cost = buy_side_cost
                }
            }

            // remove highest_priority_order from orderbook if completely satisfied
            if highest_priority_order.quantity == Decimal::ZERO {
                self.order_index.remove(&highest_priority_order.id);

                drop(highest_priority_order);
                match side {
                    Side::Sell => self.buy_side.pop_highest_priority(),
                    Side::Buy => self.sell_side.pop_highest_priority(),
                }
            }

            // add to result vec
            order_match_vec.push(highest_priority_order_order_match);
        }

        // add to result vec if not empty
        if !new_order_order_match.quantity.is_zero() {
            order_match_vec.push(new_order_order_match);
        }

        // add order to data structures if any remaining quantity
        if !quantity.is_zero() {
            let shared_order = Rc::new(RefCell::new(Order {
                id,
                side,
                price,
                quantity,
                priority: self.get_next_priority(),
            }));

            self.order_index.insert(id, shared_order.clone());
            match side {
                Side::Buy => self.buy_side.add(shared_order),
                Side::Sell => self.sell_side.add(shared_order),
            }
        }

        Ok(order_match_vec)
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
    pub fn cancel_order(&mut self, id: OrderID) -> Result<(), errors::CancelOrder> {
        let Some(shared_order) = self.order_index.remove(&id) else {
            return Err(errors::CancelOrder::OrderNotFound);
        };

        let side = shared_order.borrow().side;
        match side {
            Side::Buy => self.buy_side.remove(shared_order),
            Side::Sell => self.sell_side.remove(shared_order),
        }

        Ok(())
    }

    /// Calculates cost to buy/sell up to quantity.
    /// This function does not mutate anything in OrderBook.
    /// The return tuple is in format (quantity_fulfilled, cost).
    /// ```
    /// use rust_ob::{
    ///     OrderBook,
    ///     Side,
    ///     errors,
    /// };
    /// use rust_decimal::Decimal;
    ///
    /// let mut ob = OrderBook::new();
    /// let _ = ob.process_limit_order(1, Side::Buy, Decimal::from(5), Decimal::from(5));
    /// let _ = ob.process_limit_order(2, Side::Buy, Decimal::from(3), Decimal::from(3));
    ///
    /// assert_eq!(ob.calculate_market_cost(Side::Sell, Decimal::from(6)).unwrap(), (Decimal::from(6), Decimal::from(-28)));
    /// assert_eq!(ob.calculate_market_cost(Side::Sell, Decimal::from(12)).unwrap(), (Decimal::from(8), Decimal::from(-34)));
    ///
    /// // possible errors
    /// assert_eq!(ob.calculate_market_cost(Side::Sell, Decimal::from(0)), Err(errors::CalculateMarketCost::NonPositiveQuantity));
    /// ```
    pub fn calculate_market_cost(
        &self,
        side: Side,
        mut quantity: Decimal,
    ) -> Result<(Decimal, Decimal), errors::CalculateMarketCost> {
        // check to ensure positive quantity
        if quantity <= Decimal::ZERO {
            return Err(errors::CalculateMarketCost::NonPositiveQuantity);
        }

        // inits
        let mut quantity_fulfilled = Decimal::ZERO;
        let mut cost = Decimal::ZERO;
        let mut opposite_side_iter = match side {
            Side::Buy => self.sell_side.iter(),
            Side::Sell => self.buy_side.iter(),
        };

        while !quantity.is_zero() {
            let shared_order = opposite_side_iter.next();
            let order = match shared_order {
                Some(val) => val.borrow(),
                None => break,
            };

            let satisfied_quantity = quantity.min(order.quantity);
            quantity -= satisfied_quantity;
            quantity_fulfilled += satisfied_quantity;

            let buy_side_cost = order.price * satisfied_quantity;
            match side {
                Side::Buy => cost += buy_side_cost,
                Side::Sell => cost -= buy_side_cost,
            }
        }

        Ok((quantity_fulfilled, cost))
    }

    /// Process new market order
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
    /// let _ = ob.process_limit_order(1, Side::Sell, Decimal::from(5), Decimal::from(5));
    /// let _ = ob.process_limit_order(2, Side::Sell, Decimal::from(3), Decimal::from(3));
    ///
    /// assert_eq!(
    ///     ob.process_market_order(3, Side::Buy, Decimal::from(6)).unwrap(),
    ///     vec![
    ///         OrderMatch {
    ///             order: 2,
    ///             quantity: Decimal::from(3),
    ///             cost: Decimal::from(-9)
    ///         },
    ///         OrderMatch {
    ///             order: 1,
    ///             quantity: Decimal::from(3),
    ///             cost: Decimal::from(-15)
    ///         },
    ///         OrderMatch {
    ///             order: 3,
    ///             quantity: Decimal::from(6),
    ///             cost: Decimal::from(24)
    ///         }
    ///     ]
    /// );
    ///
    /// // possible errors
    /// assert_eq!(ob.process_market_order(4, Side::Buy, Decimal::from(0)), Err(errors::ProcessMarketOrder::NonPositiveQuantity));
    /// assert_eq!(ob.process_market_order(1, Side::Buy, Decimal::from(3)), Err(errors::ProcessMarketOrder::OrderAlreadyExists));
    /// ```
    pub fn process_market_order(
        &mut self,
        id: OrderID,
        side: Side,
        quantity: Decimal,
    ) -> Result<Vec<OrderMatch<OrderID>>, errors::ProcessMarketOrder> {
        // get min or max price based on side
        let price = match side {
            Side::Buy => Decimal::MAX,
            Side::Sell => Decimal::MIN,
        };

        let result = self
            .process_limit_order(id, side, price, quantity)
            .map_err(|e| match e {
                errors::ProcessLimitOrder::NonPositiveQuantity => {
                    errors::ProcessMarketOrder::NonPositiveQuantity
                }
                errors::ProcessLimitOrder::OrderAlreadyExists => {
                    errors::ProcessMarketOrder::OrderAlreadyExists
                }
            });

        if let Ok(ref order_match_vec) = result {
            if order_match_vec.len() == 0 || order_match_vec.last().unwrap().quantity != quantity {
                assert_eq!(self.cancel_order(id), Ok(()));
            }
        }

        result
    }

    /// Returns the `OrderID` of the next to be fulfilled order by side
    pub fn get_highest_priority_order(&self, side: Side) -> Option<OrderID> {
        let shared_order = match side {
            Side::Buy => self.buy_side.get_highest_priority(),
            Side::Sell => self.sell_side.get_highest_priority(),
        };

        shared_order.map(|o| o.borrow().id)
    }

    /// Returns the price of the next to be fulfilled order by side
    pub fn get_highest_priority_price(&self, side: Side) -> Option<Decimal> {
        let shared_order = match side {
            Side::Buy => self.buy_side.get_highest_priority(),
            Side::Sell => self.sell_side.get_highest_priority(),
        };

        shared_order.map(|o| o.borrow().price)
    }

    /// Returns (price, quantity_at_price) of the highest priority price by side
    pub fn get_highest_priority_price_quantity(&self, side: Side) -> Option<(Decimal, Decimal)> {
        // return vars
        let mut price = Decimal::ZERO;
        let mut quantity_at_price = Decimal::ZERO;

        let side_iter = match side {
            Side::Buy => self.buy_side.iter(),
            Side::Sell => self.sell_side.iter(),
        };

        for (i, order) in side_iter.map(|o| o.borrow()).enumerate() {
            if i == 0 {
                price = order.price;
            } else if price != order.price {
                break;
            }

            quantity_at_price += order.quantity
        }

        if quantity_at_price.is_zero() {
            return None;
        }

        Some((price, quantity_at_price))
    }

    fn get_next_priority(&mut self) -> u64 {
        self.priority += 1;
        self.priority
    }
}

impl<OrderID: Display> Display for OrderBook<OrderID>
where
    OrderID: Copy + PartialEq + Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const PADDING: usize = 18;
        writeln!(
            f,
            "{:->PADDING$}{:->PADDING$}{:->PADDING$}{:->PADDING$}",
            "ID", "SIDE", "PRICE", "QUANTITY"
        )?;

        for shared_order in self.sell_side.iter().rev().chain(self.buy_side.iter()) {
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

unsafe impl<OrderID: Copy + PartialEq + Eq + Hash + Send> Send for OrderBook<OrderID> {}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderMatch<OrderID> {
    /// ID of order
    pub order: OrderID,
    /// Quantity of order just fulfilled
    /// - Always positive
    pub quantity: Decimal,
    /// Cost to buy/sell quantity
    /// - Positive priced buys add to cost
    /// - Positive priced sells subract from cost
    /// - Negatively priced buys subract from cost
    /// - Negatively priced sell add to cost
    pub cost: Decimal,
}

impl<OrderID> OrderMatch<OrderID> {
    fn new(order: OrderID) -> Self {
        OrderMatch {
            order,
            quantity: Decimal::ZERO,
            cost: Decimal::ZERO,
        }
    }
}
