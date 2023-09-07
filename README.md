# rust-web-app
a rust  web app by use axum crate

## Dev environment
### install the watch by cargo install cargo-watch
```
# Terminal1 - to run the server
cargo watch -q -c -w src/ -w .cargo/ -x run 

# Terminal2 - to run the quick_start
cargo watch -q -c -w examples/ -x "run --example quick_dev"
```
