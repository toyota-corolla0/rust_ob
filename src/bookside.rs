use std::{cell::RefCell, cmp::Ordering, collections::BTreeMap, marker::PhantomData, rc::Rc};

use rust_decimal::Decimal;

use crate::order::Order;

#[derive(Debug)]
pub struct BookSide<Ordering, OrderID>
where
    BookSideKey<Ordering>: Ord,
{
    tree: BTreeMap<BookSideKey<Ordering>, Rc<RefCell<Order<OrderID>>>>,
}

impl<Priority, OrderID> BookSide<Priority, OrderID>
where
    BookSideKey<Priority>: Ord,
{
    pub fn new() -> Self {
        BookSide {
            tree: BTreeMap::new(),
        }
    }

    /// no duplicate order check present
    pub fn add(&mut self, shared_order: Rc<RefCell<Order<OrderID>>>) {
        // get map key
        let key;
        {
            let order = shared_order.borrow();
            key = BookSideKey::new(order.price, order.priority);
        }

        self.tree.insert(key, shared_order);
    }

    /// does not panic if order can't be found
    pub fn remove(&mut self, shared_order: Rc<RefCell<Order<OrderID>>>) {
        let order = shared_order.borrow();
        let key = BookSideKey::new(order.price, order.priority);

        self.tree.remove(&key);
    }

    pub fn get_highest_priority(&self) -> Option<&Rc<RefCell<Order<OrderID>>>> {
        self.tree
            .first_key_value()
            .map(|(_, shared_order)| shared_order)
    }

    /// does not panic if there is no order to pop
    pub fn pop_highest_priority(&mut self) {
        self.tree.pop_first();
    }

    pub fn iter<'a>(
        &'a self,
    ) -> Box<dyn DoubleEndedIterator<Item = &Rc<RefCell<Order<OrderID>>>> + 'a> {
        Box::new(self.tree.iter().map(|(_, a)| a))
    }
}

#[derive(Debug, Clone)]
pub struct BookSideKey<Priority> {
    price: Decimal,
    priority: u64,

    _marker: PhantomData<Priority>,
}

impl<Priority> BookSideKey<Priority> {
    fn new(price: Decimal, priority: u64) -> Self {
        BookSideKey {
            price,
            priority,
            _marker: PhantomData,
        }
    }
}
impl<Priority> PartialEq for BookSideKey<Priority> {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.priority == other.priority
    }
}
impl<Priority> PartialOrd for BookSideKey<Priority>
where
    Self: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<Priority> Eq for BookSideKey<Priority> {}

#[derive(Debug)]
pub struct MinPricePriority;
#[derive(Debug)]
pub struct MaxPricePriority;

impl Ord for BookSideKey<MinPricePriority> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.cmp(&other.price) {
            Ordering::Equal => self.priority.cmp(&other.priority),
            ord => ord,
        }
    }
}
impl Ord for BookSideKey<MaxPricePriority> {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.price.cmp(&self.price) {
            Ordering::Equal => self.priority.cmp(&other.priority),
            ord => ord,
        }
    }
}
