# WASI gRPC server helper for Rust

You can serve gRPC over wasi-http using `tonic` and `wasi-hyperium`. This crate provides a `#[grpc_component] macro to hide some of the plumbing.  Example usage:

```rust
pub mod route_guide {
    tonic::include_proto!("routeguide");
}

use route_guide::route_guide_server::RouteGuide;
use route_guide::route_guide_server::RouteGuideServer;

#[wasi_grpc_server::grpc_component(RouteGuideServer)]
struct MyRouteGuide;

#[tonic::async_trait]
impl RouteGuide for MyRouteGuide {
    // ... service implementation here ...
}
```

**IMPORTANT:** You must add the `wasi` and `wasi-hyperium` references to your server crate. The macro doesn't do this for you. You'll also need to do the usual `tonic` generation shenanigans (`tonic_build::configure()` in `build.rs`, and `tonic::include_proto!` in your server). The macro _only_ handles wiring up the `wasi:http` interface to the `tonic` gRPC handlers.

See `routeguide-server` in the `examples` folder for more.

## Status

This crate has received the Works On My Machine seal of quality.  Please expect issues and raise them on the GitHub repo.

## Known limitations

* Streaming responses do not stream: instead they are delivered as a block when the stream completes.
