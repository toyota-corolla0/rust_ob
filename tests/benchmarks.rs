use rand::prelude::*;
use rust_ob::{Decimal, OrderBook, Side};
use std::time::Instant;

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

    let time_in_nanos = start.elapsed().as_nanos();

    println!("-----PROCESS LIMIT ORDER BENCHMARK-----");
    println!(
        "Iterations: {ITERATIONS} \nTime: {time_in_nanos}ns \nAverage Iteration Time: {}ns \n",
        time_in_nanos / ITERATIONS
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

    let time_in_nanos = start.elapsed().as_nanos();

    println!("-----CANCEL ORDER BENCHMARK-----");
    println!(
        "Iterations: {ITERATIONS} \nTime: {time_in_nanos}ns \nAverage Iteration Time: {}ns \n",
        time_in_nanos / ITERATIONS
    );
}
