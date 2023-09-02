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

    let order_match_vec = match result {
        Err(Error::OrderAlreadyExists) => {
            // handle error
            panic!()
        }
        Err(Error::NonPositiveQuantity) => {
            // handle error
            panic!()
        }
        Ok(v) => v,
        _ => panic!("should never get here")
    };

    for order_match in order_match_vec {
        // handle matches
        // more information about order_match_vec can be found in documentation for OrderBook::process_limit_order
    }

    // cancel order
    let result = ob.cancel_order(1);
    
    match result {
        Some(Error::OrderNotFound) => {
            // handle error
            panic!()
        }
        None => {}
        _ => panic!("should never get here")
    }
}
```