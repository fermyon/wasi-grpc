use hyperium::{Uri, uri::Parts};
use spin_executor::CancelToken;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tonic::body::Body;
use tower_service::Service;
use wasi::io::poll::Pollable;
use wasi_hyperium::{IncomingHttpBody, hyperium1::send_outbound_request, poll::PollableRegistry};

pub struct WasiGrpcEndpoint {
    endpoint: Uri,
}

impl WasiGrpcEndpoint {
    pub fn new(endpoint: Uri) -> Self {
        WasiGrpcEndpoint { endpoint }
    }
}

impl Service<hyperium::Request<Body>> for WasiGrpcEndpoint {
    type Response = hyperium::Response<IncomingHttpBody<SpinExecutorPoller>>;
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

        *req.uri_mut() = parts.try_into().unwrap();

        Box::pin(send_outbound_request(req, SpinExecutorPoller))
    }
}

#[derive(Clone)]
pub struct SpinExecutorPoller;

impl PollableRegistry for SpinExecutorPoller {
    type RegisteredPollable = CancelToken;

    fn register_pollable(&self, cx: &mut Context, pollable: Pollable) -> Self::RegisteredPollable {
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
