use hyperium::{Uri, uri::Parts};
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;
use tonic::body::Body;
use tower_service::Service;
use wasi_hyperium::{IncomingHttpBody, hyperium1::send_outbound_request, poll::PollableRegistry};

#[cfg(feature = "spin")]
pub use spin::SpinExecutorPoller;

/// A `tower::Service` that sends tonic requests over `wasi:http`.
///
/// The `Registry` decides how WASI pollables are driven. With the default
/// `spin` feature it is [`SpinExecutorPoller`] (use with `spin_executor::run`
/// or a Spin async handler). Other executors can supply their own
/// [`PollableRegistry`] — e.g. [`wasi_hyperium::poll::Poller`] driven by its
/// own `block_on` — via [`WasiGrpcEndpoint::with_registry`].
#[cfg(feature = "spin")]
pub struct WasiGrpcEndpoint<Registry = SpinExecutorPoller> {
    endpoint: Uri,
    registry: Registry,
}

/// A `tower::Service` that sends tonic requests over `wasi:http`.
///
/// The `Registry` decides how WASI pollables are driven, e.g.
/// [`wasi_hyperium::poll::Poller`] driven by its own `block_on`.
#[cfg(not(feature = "spin"))]
pub struct WasiGrpcEndpoint<Registry> {
    endpoint: Uri,
    registry: Registry,
}

#[cfg(feature = "spin")]
impl WasiGrpcEndpoint {
    /// Creates an endpoint driven by the Spin executor.
    pub fn new(endpoint: Uri) -> Self {
        Self::with_registry(endpoint, SpinExecutorPoller)
    }
}

impl<Registry> WasiGrpcEndpoint<Registry> {
    /// Creates an endpoint driven by the given pollable registry.
    pub fn with_registry(endpoint: Uri, registry: Registry) -> Self {
        WasiGrpcEndpoint { endpoint, registry }
    }
}

impl<Registry> Service<hyperium::Request<Body>> for WasiGrpcEndpoint<Registry>
where
    Registry: PollableRegistry + Send + 'static,
    Registry::RegisteredPollable: Send,
{
    type Response = hyperium::Response<IncomingHttpBody<Registry>>;
    type Error = wasi_hyperium::Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: hyperium::Request<Body>) -> Self::Future {
        let Parts {
            scheme, authority, ..
        } = self.endpoint.clone().into_parts();

        let mut parts = std::mem::take(req.uri_mut()).into_parts();
        parts.authority = authority;
        parts.scheme = scheme;

        let uri = match Uri::from_parts(parts) {
            Ok(uri) => uri,
            Err(e) => {
                return Box::pin(std::future::ready(Err(wasi_hyperium::Error::WasiError(
                    format!("invalid gRPC request URI: {e}"),
                ))));
            }
        };
        *req.uri_mut() = uri;

        Box::pin(send_outbound_request(req, self.registry.clone()))
    }
}

#[cfg(feature = "spin")]
mod spin {
    use spin_executor::CancelToken;
    use std::task::Context;
    use wasi::io::poll::Pollable;
    use wasi_hyperium::poll::PollableRegistry;

    /// Registers WASI pollables with the Spin executor (`spin_executor::run`).
    #[derive(Clone)]
    pub struct SpinExecutorPoller;

    impl PollableRegistry for SpinExecutorPoller {
        type RegisteredPollable = CancelToken;

        fn register_pollable(
            &self,
            cx: &mut Context,
            pollable: Pollable,
        ) -> Self::RegisteredPollable {
            spin_executor::push_waker_and_get_token(pollable, cx.waker().clone())
        }

        // This should never be called when using `spin_executor::run`
        fn poll(&self) -> bool {
            panic!("not supported for wasi-grpc")
        }

        // This should never be called when using `spin_executor::run`
        fn block_on<T>(
            &self,
            _fut: impl std::future::Future<Output = T>,
        ) -> Result<T, wasi_hyperium::poll::Stalled> {
            panic!("not supported for wasi-grpc")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_grpc_service<S: Service<hyperium::Request<Body>>>() {}

    #[test]
    fn endpoint_is_a_service_for_any_registry() {
        assert_grpc_service::<WasiGrpcEndpoint<wasi_hyperium::poll::Poller>>();
        #[cfg(feature = "spin")]
        assert_grpc_service::<WasiGrpcEndpoint>();
    }
}
