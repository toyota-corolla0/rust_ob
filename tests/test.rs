use std::time::Instant;

use rust_decimal::Decimal;
use rust_ob::{errors, OrderBook, OrderMatch, Side};

use rand::prelude::*;

#[test]
fn process_limit_order1() {
    let mut ob = OrderBook::new();

    let res = ob.process_limit_order(1, Side::Buy, Decimal::from(10), Decimal::from(0));
    assert_eq!(
        res.unwrap_err(),
        errors::ProcessLimitOrder::NonPositiveQuantity
    );

    let _ = ob.process_limit_order(500, Side::Buy, Decimal::from(10), Decimal::from(10));
    let res = ob.process_limit_order(500, Side::Buy, Decimal::from(10), Decimal::from(10));
    assert_eq!(
        res.unwrap_err(),
        errors::ProcessLimitOrder::OrderAlreadyExists
    );
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
    assert_eq!(
        ob.cancel_order(9943),
        Err(errors::CancelOrder::OrderNotFound)
    );
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
fn find_market_cost1() {
    let ob = OrderBook::new();

    assert_eq!(
        ob.find_market_cost(Side::Buy, Decimal::from(0))
            .unwrap_err(),
        errors::FindMarketCost::NonPositiveQuantity
    );
    assert_eq!(
        ob.find_market_cost(Side::Buy, Decimal::from(-1))
            .unwrap_err(),
        errors::FindMarketCost::NonPositiveQuantity
    );
}

#[test]
fn find_market_cost2() {
    let mut ob = OrderBook::new();

    assert_eq!(
        ob.process_limit_order(1, Side::Buy, Decimal::from(20), Decimal::from(5))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(2, Side::Buy, Decimal::from(15), Decimal::from(3))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(3, Side::Sell, Decimal::from(35), Decimal::from(10))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(4, Side::Sell, Decimal::from(50), Decimal::from(4))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(5, Side::Sell, Decimal::from(30), Decimal::from(15))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(6, Side::Buy, Decimal::from(20), Decimal::from(2))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(7, Side::Sell, Decimal::from(35), Decimal::from(7))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(8, Side::Buy, Decimal::from(15), Decimal::from(9))
            .unwrap()
            .len(),
        0
    );

    println!("{ob}");

    assert_eq!(
        ob.find_market_cost(Side::Sell, Decimal::from(17)).unwrap(),
        (Decimal::from(17), Decimal::from(-290))
    );

    assert_eq!(
        ob.find_market_cost(Side::Buy, Decimal::from(55)).unwrap(),
        (Decimal::from(36), Decimal::from(450 + 350 + 245 + 200))
    );
}

#[test]
fn general1() {
    let mut ob = OrderBook::new();

    assert_eq!(
        ob.process_limit_order(1, Side::Buy, Decimal::from(20), Decimal::from(5))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(2, Side::Buy, Decimal::from(15), Decimal::from(3))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(3, Side::Sell, Decimal::from(35), Decimal::from(10))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(4, Side::Sell, Decimal::from(50), Decimal::from(4))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(5, Side::Sell, Decimal::from(30), Decimal::from(15))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(6, Side::Buy, Decimal::from(20), Decimal::from(2))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(7, Side::Sell, Decimal::from(35), Decimal::from(7))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(8, Side::Buy, Decimal::from(15), Decimal::from(9))
            .unwrap()
            .len(),
        0
    );

    assert_eq!(
        ob.process_limit_order(9, Side::Buy, Decimal::from(33), Decimal::from(22))
            .unwrap(),
        vec![
            OrderMatch {
                order: 5,
                quantity: Decimal::from(15),
                cost: Decimal::from(-450)
            },
            OrderMatch {
                order: 9,
                quantity: Decimal::from(15),
                cost: Decimal::from(450)
            },
        ]
    );

    assert_eq!(
        ob.process_limit_order(10, Side::Sell, Decimal::from(9), Decimal::from(18))
            .unwrap(),
        vec![
            OrderMatch {
                order: 9,
                quantity: Decimal::from(7),
                cost: Decimal::from(231)
            },
            OrderMatch {
                order: 1,
                quantity: Decimal::from(5),
                cost: Decimal::from(100)
            },
            OrderMatch {
                order: 6,
                quantity: Decimal::from(2),
                cost: Decimal::from(40)
            },
            OrderMatch {
                order: 2,
                quantity: Decimal::from(3),
                cost: Decimal::from(45)
            },
            OrderMatch {
                order: 8,
                quantity: Decimal::from(1),
                cost: Decimal::from(15)
            },
            OrderMatch {
                order: 10,
                quantity: Decimal::from(18),
                cost: Decimal::from(-231 - 100 - 40 - 45 - 15)
            },
        ]
    );

    assert_eq!(
        ob.process_limit_order(11, Side::Buy, Decimal::from(-5), Decimal::from(4))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        ob.process_limit_order(12, Side::Buy, Decimal::from(-10), Decimal::from(14))
            .unwrap()
            .len(),
        0
    );

    assert_eq!(ob.cancel_order(4), Ok(()));
    assert_eq!(ob.cancel_order(4), Err(errors::CancelOrder::OrderNotFound));

    assert_eq!(
        ob.process_limit_order(13, Side::Buy, Decimal::from(38), Decimal::from(25))
            .unwrap(),
        vec![
            OrderMatch {
                order: 3,
                quantity: Decimal::from(10),
                cost: Decimal::from(-350)
            },
            OrderMatch {
                order: 7,
                quantity: Decimal::from(7),
                cost: Decimal::from(-245)
            },
            OrderMatch {
                order: 13,
                quantity: Decimal::from(17),
                cost: Decimal::from(595)
            },
        ]
    );

    assert_eq!(
        ob.process_limit_order(14, Side::Sell, Decimal::from(-17), Decimal::from(35))
            .unwrap(),
        vec![
            OrderMatch {
                order: 13,
                quantity: Decimal::from(8),
                cost: Decimal::from(304)
            },
            OrderMatch {
                order: 8,
                quantity: Decimal::from(8),
                cost: Decimal::from(120)
            },
            OrderMatch {
                order: 11,
                quantity: Decimal::from(4),
                cost: Decimal::from(-20)
            },
            OrderMatch {
                order: 12,
                quantity: Decimal::from(14),
                cost: Decimal::from(-140)
            },
            OrderMatch {
                order: 14,
                quantity: Decimal::from(34),
                cost: Decimal::from(-264)
            },
        ]
    );

    assert_eq!(
        ob.process_limit_order(15, Side::Buy, Decimal::from(33), Decimal::from(1))
            .unwrap(),
        vec![
            OrderMatch {
                order: 14,
                quantity: Decimal::from(1),
                cost: Decimal::from(17)
            },
            OrderMatch {
                order: 15,
                quantity: Decimal::from(1),
                cost: Decimal::from(-17)
            },
        ]
    );
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
