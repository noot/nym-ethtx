# Ethereum transaction submitter using Nym

This library provides functionality for submitting Ethereum transactions using the Nym mixnet.

It has two components:
- `client`: sends transactions to submit to a `server` over the Nym network
- `server`: recieves transactions from `client`s and submits them to Ethereum

## Requirements

- rust 1.25.1
- Nym (see build instructions [here](https://nymtech.net/docs/stable/run-nym-nodes/build-nym)).

## Usage

### Run Nym websockets client

Both the client and server require a Nym websockets client to connect to.

To run one, go to your `nym` dir:
```
./target/release/nym-client init --id <your-id>
./target/release/nym-client run --id <your-id>
```

### Build

```bash
git clone https://github.com/noot/nym-ethtx.git
cd nym-ethtx
cargo build --release
```

### Network

By default, this library uses a local Ethereum chain (`http://localhost:8545`). However, it also supports mainnet and Goerli with the `--network` or `-n` flags.

### Client

The client by default uses the server with Nym address `HGLX5467Kr8hHaYENr8meY3KDH5BozVQRR8XTBD8UseB.Fdnv3igmSrGcUZSA4bUyqa6adyHKjZGyhFnnkWMJsGAt@62Lq9D5yhRVXyeHrBjqoQMg3i9aVTJY7nQSnB74VH31t`. If this isn't available, you'll need to run a `server` or find one to send the request to.

The client requires a (hex-encoded) Ethereum private key to sign the transaction. The app looks for this key by default in the file `client.key`. You can also specify the file with `--key` or `-k`.
 
For example, to send ether:
```bash
./target/release/client --to=0x1EA777Dc621f5A63E63bbcE4fc9caE3c5CDEDAFB --value=0.1 --network=goerli
# Sep 24 18:37:37.106  INFO client: signed transaction 0xbe292c8e43d775ef8ec58f974e6403317efdcdc0a09ed8232238dd8bea44ac10
```

The `to` address can also be an ENS address:
```bash
./target/release/client --to=address.eth --value=0.1 --network=goerli
# Sep 24 18:37:37.106  INFO client: signed transaction 0x0958489d9ab18796439ed38c8028421899125d11fd6ef917118343b5ab7370e3
```

For more transaction options, see `./target/release/client --help`.

### Server

```bash
./target/release/server
# Sep 24 18:41:28.450  INFO server: listening on DXHLCASnJGSesso5hXus1CtgifBpaPqAj7thZphp52xN.7udbVvZ199futJNur71L3vHDNdnbVxxBvFKVzhEifXvE@5vC8spDvw5VDQ8Zvd9fVvBhbUDv9jABR4cXzd4Kh5vz
```

For more options, see `./target/release/server --help`.
