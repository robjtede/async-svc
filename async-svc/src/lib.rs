//! WIP Modern Service Trait

// #![no_std]
#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, nonstandard_style)]
#![allow(incomplete_features)]
#![feature(type_alias_impl_trait, associated_type_defaults)]

// extern crate alloc;

// use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

// mod boxed;
// mod ext;
// mod factory;
// mod fn_factory;
mod fn_service;
// mod map;
// mod then;

// pub use boxed::{box_svc, BoxFut, BoxSvc};
// pub use ext::SvcExt;
// pub use factory::SvcFactory;
// pub use fn_factory::{fn_factory, FnFactory};
pub use self::fn_service::{fn_service, FnSvc};
// pub use self::map::MapSvc;
// pub use then::ThenSvc;

/// Service trait representing an asynchronous request/response operation.
pub trait Svc<Req> {
    /// Output type.
    type Res;

    /// Response future.
    type Fut<'fut>: Future<Output = Self::Res>
    where
        Self: 'fut;

    /// Processes request, producing a future that outputs the response type.
    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut<'_>;
}

impl<S, Req> Svc<Req> for Pin<Box<S>>
where
    S: Svc<Req>,
{
    type Res = S::Res;
    type Fut<'fut> = S::Fut<'fut> where Self: 'fut;

    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut<'_> {
        self.get_mut().as_mut().exec(req)
    }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;

    use super::*;

    struct ReturnsAsyncFn(String);

    impl Svc<()> for ReturnsAsyncFn {
        type Res = usize;

        type Fut<'fut> = impl Future<Output = Self::Res> + 'fut where Self: 'fut;

        fn exec(self: Pin<&mut Self>, _req: ()) -> Self::Fut<'_> {
            async move {
                let a = self.0.as_bytes();
                a.len()
            }
        }
    }

    #[tokio::test]
    async fn async_svc_that_borrows() {
        let svc = ReturnsAsyncFn("".to_owned());
        assert_eq!(pin!(svc).exec(()).await, 0);

        let svc = ReturnsAsyncFn("foo".to_owned());
        assert_eq!(pin!(svc).exec(()).await, 3);
    }
}
