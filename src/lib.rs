//! WIP Modern Service Trait

#![deny(rust_2018_idioms, nonstandard_style)]
// #![warn(missing_docs)]

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

mod boxed;
mod ext;
mod factory;
mod fn_service;
mod map;
mod then;

pub use boxed::{box_svc, BoxFut, BoxSvc};
pub use ext::SvcExt;
pub use fn_service::FnSvc;
pub use map::MapSvc;
pub use then::ThenSvc;

/// Service trait representing an asynchronous request/response operation.
pub trait Svc<Req> {
    /// Output type.
    type Res;

    /// Response future.
    type Fut: Future<Output = Self::Res>;

    /// To be called before `exec` to signal wether the service is ready to process requests.
    /// As such, the check should be inexpensive. Returning `Poll::Pending` acts as back-pressure.
    /// The default implementation unconditionally indicates the service is ready.
    #[allow(unused_variables)]
    fn poll_ready(self: Pin<&mut Self>, cx: Context<'_>) -> Poll<()> {
        Poll::Ready(())
    }

    /// Processes request, producing a future that outputs the response type.
    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut;
}

impl<S, Req> Svc<Req> for Box<S>
where
    S: Svc<Req>,
{
    type Res = S::Res;
    type Fut = S::Fut;

    fn poll_ready(mut self: Pin<&mut Self>, cx: Context<'_>) -> Poll<()> {
        self.as_mut().poll_ready(cx)
    }

    fn exec(mut self: Pin<&mut Self>, req: Req) -> Self::Fut {
        self.as_mut().exec(req)
    }
}
