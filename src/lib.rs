//! WIP Modern Service Trait

#![no_std]
// #![forbid(unsafe_code)]
#![deny(rust_2018_idioms, nonstandard_style)]
#![allow(incomplete_features)]
#![feature(generic_associated_types, type_alias_impl_trait)]

extern crate alloc;

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

// mod boxed;
mod ext;
mod factory;
mod fn_factory;
mod fn_service;
mod map;
mod then;

// pub use boxed::{box_svc, BoxFut, BoxSvc};
pub use ext::SvcExt;
pub use factory::SvcFactory;
pub use fn_factory::{fn_factory, FnFactory};
pub use fn_service::{fn_service, FnSvc};
pub use map::MapSvc;
pub use then::ThenSvc;

/// Service trait representing an asynchronous request/response operation.
pub trait Svc<Req> {
    /// Output type.
    type Res;

    /// Response future.
    type Fut<'fut>: Future<Output = Self::Res>;

    /// To be called before `exec` to signal whether the service is ready to process requests.
    /// As such, the check should be inexpensive. Returning `Poll::Pending` acts as back-pressure.
    /// The default implementation unconditionally indicates the service is ready.
    #[allow(unused_variables)]
    fn poll_ready(self: Pin<&mut Self>, cx: Context<'_>) -> Poll<()> {
        Poll::Ready(())
    }

    /// Processes request, producing a future that outputs the response type.
    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut<'_>;
}

impl<S, Req> Svc<Req> for Pin<Box<S>>
where
    S: Svc<Req>,
{
    type Res = S::Res;
    type Fut<'fut> = S::Fut<'fut>;

    fn poll_ready(self: Pin<&mut Self>, cx: Context<'_>) -> Poll<()> {
        self.get_mut().as_mut().poll_ready(cx)
    }

    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut<'_> {
        self.get_mut().as_mut().exec(req)
    }
}
