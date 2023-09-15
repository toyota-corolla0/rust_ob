use std::{cell::RefCell, cmp::Ordering, collections::BTreeMap, rc::Rc};

use rust_decimal::Decimal;

use crate::order::Order;

#[derive(Debug)]
pub struct BookSide<K>
where
    K: Key,
{
    price_tree: BTreeMap<K, Rc<RefCell<Order>>>,
}

impl<K> BookSide<K>
where
    K: Key,
{
    pub fn new() -> Self {
        BookSide {
            price_tree: BTreeMap::new(),
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
        self.price_tree
            .first_key_value()
            .map(|(_, shared_order)| shared_order)
    }

    /// does not panic if there is no order to pop
    pub fn pop_highest_priority(&mut self) {
        self.price_tree.pop_first();
    }

    pub fn iter<'a>(&'a self) -> Box<dyn DoubleEndedIterator<Item = &Rc<RefCell<Order>>> + 'a> {
        Box::new(self.price_tree.iter().map(|(_, a)| a))
    }
}

pub trait Key: Ord {
    fn new(price: Decimal, priority: u64) -> Self;
}

#[derive(Debug, Clone)]
pub struct MinPricePriority {
    price: Decimal,
    priority: u64,
}

impl Key for MinPricePriority {
    fn new(price: Decimal, priority: u64) -> Self {
        MinPricePriority { price, priority }
    }
}
impl PartialEq for MinPricePriority {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.priority == other.priority
    }
}
impl PartialOrd for MinPricePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.price.partial_cmp(&other.price) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.priority.partial_cmp(&other.priority)
    }
}
impl Eq for MinPricePriority {}
impl Ord for MinPricePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.priority.cmp(&other.priority)
    }
}

#[derive(Debug, Clone)]
pub struct MaxPricePriority {
    price: Decimal,
    priority: u64,
}

impl Key for MaxPricePriority {
    fn new(price: Decimal, priority: u64) -> Self {
        MaxPricePriority {
            price,
            priority,
        }
    }
}
impl PartialEq for MaxPricePriority {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.priority == other.priority
    }
}
impl PartialOrd for MaxPricePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match other.price.partial_cmp(&self.price) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.priority.partial_cmp(&other.priority)
    }
}
impl Eq for MaxPricePriority {}
impl Ord for MaxPricePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.price.cmp(&self.price) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.priority.cmp(&other.priority)
    }
}
