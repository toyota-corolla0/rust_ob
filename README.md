# rust_ob
An orderbook library for rust

# usage
```rust
use rust_decimal::Decimal;
use rust_ob::{OrderBook, Side, Error};

fn main() {
    let mut ob = OrderBook::new();

    // create limit order
    let result = ob.process_limit_order(1, Side::Buy, Decimal::from(10), Decimal::from(10));
    if let Err(Error::OrderAlreadyExists) = result {
        // handle error
    }
    if let Err(Error::NonPositiveQuantity) = result {
        // handle error
    }

    if result.is_err() {
        panic!("should never get here")
    }

    let order_match_vec = result.unwrap();

    for order_match in order_match_vec {
        // handle matches
        // more information about order_match_vec can be found in documentation for OrderBook::process_limit_order
    }

    // cancel limit order
    let result = ob.cancel_order(1);
    if let Some(Error::OrderNotFound) = result {
        // handle error
    }
    if result.is_some() {
        panic!("should never get here")
    } 
}
```