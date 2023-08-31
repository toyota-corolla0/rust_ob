use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use rb_tree::{RBMap, rbmap::Iter};
use rust_decimal::Decimal;

use crate::order::Order;

#[derive(Debug)]
pub struct BookSide<K>
where
    K: RBMapKey,
{
    price_tree: RBMap<K, Rc<RefCell<Order>>>,
}

impl<K> BookSide<K>
where
    K: RBMapKey,
{
    pub fn new() -> Self {
        BookSide {
            price_tree: RBMap::new(),
        }
    }

    /// no duplicate order check present
    pub fn add(&mut self, shared_order: Rc<RefCell<Order>>) {
        // get map key
        let key;
        {
            let order = shared_order.borrow();
            key = K::new(order.price, order.priority);
        }

        self.price_tree.insert(key, shared_order);
    }

    /// does not panic if order can't be found
    pub fn remove(&mut self, shared_order: Rc<RefCell<Order>>) {
        let key;

        {
            let order = shared_order.borrow();
            key = K::new(order.price, order.priority);
        }

        self.price_tree.remove(&key);
    }

    pub fn get_highest_priority(&self) -> Option<&Rc<RefCell<Order>>> {
        self.price_tree.peek()
    }

    /// does not panic if there is no order to pop
    pub fn pop_highest_priority(&mut self) {
        self.price_tree.pop();
    }

    pub fn iter(&self) -> Iter<'_, K, Rc<RefCell<Order>>> {
        self.price_tree.iter()
    }
}

pub trait RBMapKey: PartialOrd + Clone {
    fn new(price: Decimal, priority: u64) -> Self;
}

#[derive(Debug, Clone)]
pub struct MinPricePriority(Decimal, u64);

impl RBMapKey for MinPricePriority {
    fn new(price: Decimal, priority: u64) -> Self {
        MinPricePriority(price, priority)
    }
}
impl PartialEq for MinPricePriority {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl PartialOrd for MinPricePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.0.partial_cmp(&other.0)  {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.1.partial_cmp(&other.1)
    }
}

#[derive(Debug, Clone)]
pub struct MaxPricePriotity(Decimal, u64);

impl RBMapKey for MaxPricePriotity {
    fn new(price: Decimal, priority: u64) -> Self {
        MaxPricePriotity(price, priority)
    }
}
impl PartialEq for MaxPricePriotity {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
impl PartialOrd for MaxPricePriotity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match other.0.partial_cmp(&self.0) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.1.partial_cmp(&other.1)
    }
}


