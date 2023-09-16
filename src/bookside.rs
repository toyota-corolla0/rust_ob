use std::{cell::RefCell, cmp::Ordering, collections::BTreeMap, marker::PhantomData, rc::Rc};

use rust_decimal::Decimal;

use crate::order::Order;

#[derive(Debug)]
pub struct BookSide<T>
where
    BookSideKey<T>: Ord,
{
    tree: BTreeMap<BookSideKey<T>, Rc<RefCell<Order>>>,
}

impl<T> BookSide<T>
where
    BookSideKey<T>: Ord,
{
    pub fn new() -> Self {
        BookSide {
            tree: BTreeMap::new(),
        }
    }

    /// no duplicate order check present
    pub fn add(&mut self, shared_order: Rc<RefCell<Order>>) {
        // get map key
        let key;
        {
            let order = shared_order.borrow();
            key = BookSideKey::new(order.price, order.priority);
        }

        self.tree.insert(key, shared_order);
    }

    /// does not panic if order can't be found
    pub fn remove(&mut self, shared_order: Rc<RefCell<Order>>) {
        let order = shared_order.borrow();
        let key = BookSideKey::new(order.price, order.priority);

        self.tree.remove(&key);
    }

    pub fn get_highest_priority(&self) -> Option<&Rc<RefCell<Order>>> {
        self.tree
            .first_key_value()
            .map(|(_, shared_order)| shared_order)
    }

    /// does not panic if there is no order to pop
    pub fn pop_highest_priority(&mut self) {
        self.tree.pop_first();
    }

    pub fn iter<'a>(&'a self) -> Box<dyn DoubleEndedIterator<Item = &Rc<RefCell<Order>>> + 'a> {
        Box::new(self.tree.iter().map(|(_, a)| a))
    }
}

#[derive(Debug, Clone)]
pub struct BookSideKey<T> {
    price: Decimal,
    priority: u64,

    _marker: PhantomData<T>,
}

impl<T> BookSideKey<T> {
    fn new(price: Decimal, priority: u64) -> Self {
        BookSideKey {
            price,
            priority,
            _marker: PhantomData,
        }
    }
}
impl<T> PartialEq for BookSideKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.priority == other.priority
    }
}
impl<T> PartialOrd for BookSideKey<T>
where
    Self: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Eq for BookSideKey<T> {}

#[derive(Debug)]
pub struct MinPricePriority;
#[derive(Debug)]
pub struct MaxPricePriority;

impl Ord for BookSideKey<MinPricePriority> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.priority.cmp(&other.priority)
    }
}
impl Ord for BookSideKey<MaxPricePriority> {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.price.cmp(&self.price) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.priority.cmp(&other.priority)
    }
}
