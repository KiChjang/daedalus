# Daedalus Payments Engine

This repository contains the code for the Daedalus payments engine. It takes in a CSV file of transaction records, and outputs the list of clients with their associated statements.

## Usage

```
daedalus transaction.csv
// or
cargo run -- transaction.csv
```

`transaction.csv` is expected to be a comma-separated list of records containing the transaction type, client ID, transaction ID and transaction amount in that order. The output of the program is also a comma-separated list of records containing the client ID, available funds, funds held by dispute, total funds, and account locked status in that order.

## Testing

```
cargo test
```

`transaction.rs` contains unit tests for deserializing transactions. Each transaction type has its own unit test for deserialization. The unit test functions all return a `csv::Result<()>` so that if anything was amiss with the deserialization process, we would get a helpful error message on what went wrong while running the unit tests.

`client.rs` contains unit tests for account actions performed on the client. The unit tests so far deal with the following cases:

1. Simple deposits and withdrawals
2. Disallowing withdrawals when there are insufficient available funds
3. Re-adds the held funds to available funds after disputes are resolved, and allow for withdrawal of that amount
4. Ensure client is locked after a chargeback occurs, and prevent all subsequent deposits and withdrawals
5. Allow for withdrawals even when client is under dispute, as long as there is sufficient available funds
6. Ensure that a withdrawal dispute is handled differently from a deposit dispute

## Nomenclature
Daedalus was a skillful Greek architect and craftsman, and was seen as a symbol of wisdom, knowledge, and power. He was famous for his warning to his unheeded warning to his son, "don't fly too close to the sun", reminding us that while blockchain and cryptocurrency has allowed us to soar into much higher heights of technological advancements, we should keep in mind of the potential pitfalls of bleeding edge tech, lest we find ourselves being burnt out by the heat and falling to our demise.
