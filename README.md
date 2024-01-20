# Rust Web API template

# Description
Plantilla de una API web con estructuras de base de datos para postgres y mongoDB

# Execution

## Run
### Terminal 1 - To run the server.
cargo run -p web-server

### Terminal 2 - To run the tests.
cargo run -p web-server --example quick_dev

## Watch
### Terminal 1 - To run the server.
cargo watch -q -c -w crates/services/web-server/src/ -w crates/libs/ -w .cargo/ -x "run -p web-server"

### Terminal 2 - To run the quick_dev.
cargo watch -q -c -w crates/services/web-server/examples/ -x "run -p web-server --example quick_dev"


# Test

## All tests
cargo test

## All model tests
cargo test model::task::test

## One test
cargo test model::activity::tests::test_delete_activity_err_not_found

# Tools
cargo run -p gen-key