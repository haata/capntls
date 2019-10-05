# capntls
rough proof of concept for asynchronous Cap'n Proto RPC over TLS

```
cargo run server localhost:3276
cargo run client localhost:3276

cargo run --example server localhost:3277
cargo run --example client localhost:3277
```

Two pycapnp are also provided
```
cargo run server localhost:3278
cd pytest
pipenv install
pipenv shell
./async_ssl_client.py localhost:3278

cargo run server localhost:3279
cd pytest
pipenv install
pipenv shell
./async_reconnecting_ssl_client.py localhost:3279
```
