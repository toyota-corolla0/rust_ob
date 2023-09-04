use std::time::Instant;

use rust_decimal::Decimal;
use rust_ob::{OrderBook, OrderMatch, Side, errors};

use rand::prelude::*;

#[test]
fn process_limit_order1() {
    let mut ob = OrderBook::new();

    let res = ob.process_limit_order(1, Side::Buy, Decimal::from(10), Decimal::from(0));
    assert_eq!(res.unwrap_err(), errors::ProcessLimitOrder::NonPositiveQuantity);

    let _ = ob.process_limit_order(500, Side::Buy, Decimal::from(10), Decimal::from(10));
    let res = ob.process_limit_order(500, Side::Buy, Decimal::from(10), Decimal::from(10));
    assert_eq!(res.unwrap_err(), errors::ProcessLimitOrder::OrderAlreadyExists);
}

#[test]
fn process_limit_order2() {
    let mut ob = OrderBook::new();

    let res1 = ob.process_limit_order(1, Side::Sell, Decimal::from(4), Decimal::from(4));
    let res2 = ob.process_limit_order(2, Side::Sell, Decimal::from(3), Decimal::from(2));
    let res3 = ob.process_limit_order(3, Side::Buy, Decimal::from(8), Decimal::from(3));

    assert_eq!(res1.unwrap().len(), 0);
    assert_eq!(res2.unwrap().len(), 0);
    assert_eq!(
        res3.unwrap(),
        vec![
            OrderMatch {
                order: 2,
                quantity: Decimal::from(2),
                cost: Decimal::from(-6)
            },
            OrderMatch {
                order: 1,
                quantity: Decimal::from(1),
                cost: Decimal::from(-4)
            },
            OrderMatch {
                order: 3,
                quantity: Decimal::from(3),
                cost: Decimal::from(10)
            }
        ]
    );
}

#[test]
fn process_limit_order3() {
    let mut ob = OrderBook::new();

    let res1 = ob.process_limit_order(1, Side::Buy, Decimal::from(5), Decimal::from(11));
    let res2 = ob.process_limit_order(2, Side::Sell, Decimal::from(3), Decimal::from(15));
    let res3 = ob.process_limit_order(3, Side::Sell, Decimal::from(3), Decimal::from(12));
    let res4 = ob.process_limit_order(4, Side::Buy, Decimal::from(4), Decimal::from(45));
    let res5 = ob.process_limit_order(5, Side::Sell, Decimal::from(4), Decimal::from(12));

    assert_eq!(res1.unwrap().len(), 0);
    assert_eq!(
        res2.unwrap(),
        vec![
            OrderMatch {
                order: 1,
                quantity: Decimal::from(11),
                cost: Decimal::from(55)
            },
            OrderMatch {
                order: 2,
                quantity: Decimal::from(11),
                cost: Decimal::from(-55)
            },
        ]
    );
    assert_eq!(res3.unwrap().len(), 0);
    assert_eq!(
        res4.unwrap(),
        vec![
            OrderMatch {
                order: 2,
                quantity: Decimal::from(4),
                cost: Decimal::from(-12)
            },
            OrderMatch {
                order: 3,
                quantity: Decimal::from(12),
                cost: Decimal::from(-36)
            },
            OrderMatch {
                order: 4,
                quantity: Decimal::from(16),
                cost: Decimal::from(48)
            },
        ]
    );
    assert_eq!(
        res5.unwrap(),
        vec![
            OrderMatch {
                order: 4,
                quantity: Decimal::from(12),
                cost: Decimal::from(48)
            },
            OrderMatch {
                order: 5,
                quantity: Decimal::from(12),
                cost: Decimal::from(-48)
            },
        ]
    );
}

#[test]
fn process_limit_order_benchmark() {
    static ITERATIONS: u128 = 10000;

    let mut ob = OrderBook::new();

    let start = Instant::now();

    for i in 0..ITERATIONS {
        let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };

        let price = Decimal::from(random::<u8>());
        let quantity = Decimal::from(random::<u8>());

        let _ = ob.process_limit_order(i, side, price, quantity);
    }

    let time_in_millis = start.elapsed().as_millis();

    println!("{ob}");
    println!("Iterations: {ITERATIONS}    Time: {time_in_millis}ms");
}

#[test]
fn cancel_order1() {
    let mut ob = OrderBook::new();
    let _ = ob.process_limit_order(884213, Side::Sell, Decimal::from(5), Decimal::from(5));

    assert_eq!(ob.cancel_order(884213), Ok(()));
    assert_eq!(ob.cancel_order(9943), Err(errors::CancelOrder::OrderNotFound));
}

#[test]
fn cancel_order_benchmark() {
    const ITERATIONS: u128 = 10000;

    let mut ob = OrderBook::new();
    for i in 0..ITERATIONS {
        let _ = ob.process_limit_order(
            i,
            Side::Sell,
            Decimal::from(random::<u16>()),
            Decimal::from(1),
        );
    }

    let start = Instant::now();

    for i in 0..ITERATIONS {
        let _ = ob.cancel_order(i);
    }

    let time_in_millis = start.elapsed().as_millis();

    println!("{ob}");
    println!("Iterations: {ITERATIONS}    Time: {time_in_millis}ms");
}

#[test]
fn print() {
    let mut ob = OrderBook::new();

    let _ = ob.process_limit_order(1, Side::Buy, Decimal::from(4), Decimal::from(4));
    let _ = ob.process_limit_order(2, Side::Buy, Decimal::from(3), Decimal::from(2));
    let _ = ob.process_limit_order(3, Side::Buy, Decimal::from(8), Decimal::from(3));
    let _ = ob.process_limit_order(4, Side::Buy, Decimal::from(3), Decimal::from(8));
    let _ = ob.process_limit_order(5, Side::Buy, Decimal::from(3), Decimal::from(3));
    let _ = ob.process_limit_order(6, Side::Sell, Decimal::from(10), Decimal::from(5));
    let _ = ob.process_limit_order(7, Side::Sell, Decimal::from(12), Decimal::from(3));

    println!("{ob}");
}
