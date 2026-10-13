# wasi-grpc

Enables **gRPC clients** to work inside **Spin** components using [`wasi-hyperium`](https://github.com/fermyon/wasi-hyperium) and the [`spin-rust-sdk`](https://github.com/fermyon/spin-rust-sdk).

Spin is a fast, secure WebAssembly framework for serverless apps. This project extends Spin's capabilities by enabling components to make outbound gRPC requests, bridging the gap between Wasm sandboxing and modern microservice communication.

## 🚀 Features

- 🧩 **gRPC over HTTP/2** support inside Spin apps
- 🔐 Compatible with the Spin sandbox security model
- 🦀 Built with `wasi-hyperium` and `tonic`
- 🌍 Works with most standard gRPC services
- 🧪 Great for calling internal microservices or public gRPC APIs from Spin

## 🛠️ Usage

### 1. Add required dependencies to your Cargo.toml

```toml
[dependencies]
anyhow = "1"
futures = "0.3.28"
prost = "0.14"
tonic-prost = "0.14"
wasi-grpc = "0.1.0"
spin-sdk = "5.2"
tonic = { version = "0.14", features = ["codegen", "router"], default-features = false }
```

`wasi-grpc` registers WASI pollables with `spin-executor` 5, so use a `spin-sdk` that is built on `spin-executor` 5 (`spin-sdk` 5.x).

### 2. Generate gRPC client code with `tonic-prost-build`

In `build.rs`:

```rust
fn main() {
    tonic_prost_build::configure()
        .build_transport(false)
        .compile_protos(&["route_guide.proto"], &[""])
        .unwrap();
}
```

And in `Cargo.toml`:

```toml
[build-dependencies]
tonic-prost-build = "0.14"
```

### 3. Call your gRPC service from a Spin component

```rust
let endpoint = WasiGrpcEndpoint::new(endpoint_uri);
let mut client = RouteGuideClient::new(endpoint);

let response = client.get_feature(Request::new(Point {
    latitude: 409_146_138,
    longitude: -746_188_906,
})).await?;
```

### Using another executor

`WasiGrpcEndpoint::new` drives WASI pollables through the Spin executor. To use another executor, pass any [`PollableRegistry`](https://docs.rs/wasi-hyperium/latest/wasi_hyperium/poll/trait.PollableRegistry.html) to `WasiGrpcEndpoint::with_registry`. For example, `wasi_hyperium::poll::Poller` with its own `block_on`:

```rust
use wasi_hyperium::poll::{Poller, PollableRegistry};

let poller = Poller::default();
let endpoint = WasiGrpcEndpoint::with_registry(endpoint_uri, poller.clone());
let mut client = RouteGuideClient::new(endpoint);
let feature = poller.block_on(client.get_feature(Request::new(point)))??;
```

Disable the default `spin` feature to drop the `spin-executor` dependency.

## 🔒 Spin Notes

This project assumes execution in a **Spin-compliant runtime** that allows outbound networking via Spin's capability-based model. Ensure your `spin.toml` includes the appropriate `allowed_outbound_hosts` granting outbound access to your gRPC service's endpoint.