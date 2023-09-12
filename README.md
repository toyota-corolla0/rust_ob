# rust_ob
An orderbook library for rust

### Features
- standard price-time priority
- execution of limit and market order
- support for negative prices

### Usage
```rust
use rust_decimal::Decimal;
use rust_ob::{errors, OrderBook, Side};

fn main() {
    // create orderbook
    let mut ob = OrderBook::new();

    // process limit order
    {
        let result = ob.process_limit_order(1, Side::Buy, Decimal::from(10), Decimal::from(10));

        use errors::ProcessLimitOrder as E;
        let order_match_vec = match result {
            Err(E::OrderAlreadyExists) => {
                // handle error
                panic!()
            }
            Err(E::NonPositiveQuantity) => {
                // handle error
                panic!()
            }
            Ok(v) => v,
        };

        for order_match in order_match_vec {
            // handle matches
            // more information about order_match_vec can be found in documentation for OrderBook::process_limit_order
        }
    }

    // cancel order
    {
        let result = ob.cancel_order(1);

        use errors::CancelOrder as E;
        match result {
            Ok(()) => {}
            Err(E::OrderNotFound) => {
                // handle error
                panic!()
            }
        }
    }
}
```