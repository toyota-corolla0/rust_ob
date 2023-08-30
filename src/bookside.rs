use std::{cell::RefCell, cmp::Ordering, collections::LinkedList, rc::Rc};

use rb_tree::RBMap;
use rust_decimal::Decimal;

use crate::order::{Order, ID};

#[derive(Debug)]
pub struct BookSide<K>
where
    K: RBMapKey,
{
    price_tree: RBMap<K, OrderQueue>,
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

    // no duplicate order check present
    pub fn add(&mut self, shared_order: Rc<RefCell<Order>>) {
        // get map key
        let key;
        {
            let order = shared_order.borrow();
            key = K::new(order.price);
        }

        // create OrderQueue for price level if it does not exist
        let order_queue = match self.price_tree.get(&key) {
            Some(val) => val,
            None => {
                self.price_tree.insert(key.clone(), OrderQueue::new());
                self.price_tree.get(&key).unwrap()
            }
        };

        order_queue.push_front(shared_order);
    }

    // panics if order cannot be found
    pub fn remove(&mut self, shared_order: Rc<RefCell<Order>>) {
        let key;
        let id;
        {
            let order = shared_order.borrow();
            key = K::new(order.price);
            id = order.id;
        }

        match self.price_tree.get(&key) {
            Some(order_queue) => {
                order_queue.remove_by_id_or_panic(id);

                if order_queue.len() == 0 {
                    self.price_tree.remove(&key);
                }
            }

            None => panic!("BookSide: remove: order queue for price level not found"),
        }
    }

    pub fn get_highest_priority(&self) -> Option<Rc<RefCell<Order>>> {
        self.price_tree.peek()?.back()
    }

    // panics if no order to pop
    pub fn pop_highest_priority(&mut self) {
        let order_queue = match self.price_tree.peek() {
            Some(val) => val,
            None => panic!("BookSide: pop_highest_priority: no OrderQueue in BookSide"),
        };

        if let None = order_queue.pop_back() {
            panic!("BookSide: pop_highest_priority: no Order in OrderQueue");
        };

        if order_queue.len() == 0 {
            self.price_tree.pop();
        }
    }

    pub fn iter(&self) -> BookSideIter {
        let mut order_queues = Vec::new();
        for (_, order_queue) in self.price_tree.iter() {
            order_queues.push(order_queue);
        }

        BookSideIter::new(order_queues)
    }
}

pub trait RBMapKey: PartialOrd + Clone {
    fn new(price: Decimal) -> Self;
}

#[derive(Debug, Clone)]
pub struct MinPricePriority(Decimal);

impl RBMapKey for MinPricePriority {
    fn new(price: Decimal) -> Self {
        MinPricePriority(price)
    }
}
impl PartialEq for MinPricePriority {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl PartialOrd for MinPricePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[derive(Debug, Clone)]
pub struct MaxPricePriotity(Decimal);

impl RBMapKey for MaxPricePriotity {
    fn new(price: Decimal) -> Self {
        MaxPricePriotity(price)
    }
}
impl PartialEq for MaxPricePriotity {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl PartialOrd for MaxPricePriotity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

// The highest priority order is at the back of LinkedList
#[derive(Debug)]
struct OrderQueue(RefCell<LinkedList<Rc<RefCell<Order>>>>);

impl OrderQueue {
    fn new() -> Self {
        OrderQueue(RefCell::new(LinkedList::new()))
    }

    fn len(&self) -> usize {
        self.0.borrow().len()
    }

    fn push_front(&self, shared_order: Rc<RefCell<Order>>) {
        self.0.borrow_mut().push_front(shared_order);
    }

    fn back(&self) -> Option<Rc<RefCell<Order>>> {
        self.0.borrow().back().map(|val| val.clone())
    }

    fn pop_back(&self) -> Option<Rc<RefCell<Order>>> {
        self.0.borrow_mut().pop_back()
    }

    fn remove_by_id_or_panic(&self, id: ID) {
        let index = self.find_index_by_id_or_panic(id);

        let mut linked_list = self.0.borrow_mut();
        let mut split_list = linked_list.split_off(index);
        split_list.pop_front();
        linked_list.append(&mut split_list);
    }

    fn find_index_by_id_or_panic(&self, id: ID) -> usize {
        for (i, shared_order) in self.0.borrow().iter().enumerate() {
            let order = shared_order.borrow();

            if id == order.id {
                return i;
            }
        }

        panic!("OrderQueue: find_index_by_id: could not find order with given id")
    }

    fn orders_reverse_priority_sorted(&self) -> Vec<Rc<RefCell<Order>>> {
        let mut result = Vec::new();
        for shared_order in self.0.borrow().iter() {
            result.push(shared_order.clone());
        }

        result
    }
}

pub struct BookSideIter<'a> {
    order_queues_priority_sorted: Vec<&'a OrderQueue>,
    next_order_queue_index: usize,
    
    orders_reverse_priority_sorted: Vec<Rc<RefCell<Order>>>,
}

impl<'a> BookSideIter<'a> {
    fn new(order_queues: Vec<&'a OrderQueue>) -> Self {
        BookSideIter { 
            order_queues_priority_sorted: order_queues,
            next_order_queue_index: 0,
            orders_reverse_priority_sorted: Vec::new(),
        }
    }
}
impl<'a> Iterator for BookSideIter<'a> {
    type Item = Rc<RefCell<Order>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.orders_reverse_priority_sorted.len() == 0 {
            self.orders_reverse_priority_sorted = self.order_queues_priority_sorted.get(self.next_order_queue_index)?.orders_reverse_priority_sorted();
            self.next_order_queue_index += 1;          
        }

        self.orders_reverse_priority_sorted.pop()
    }
}
