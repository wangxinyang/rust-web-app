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

## Starting the DB
```
# Start postgresql server docker image:
docker run --rm --name pg -p 5432:5432 \
   -e POSTGRES_PASSWORD=welcome \
   postgres:15
```
