//! WIP Modern Service Trait

// #![no_std]
#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, nonstandard_style)]
#![allow(incomplete_features)]
#![feature(type_alias_impl_trait, associated_type_defaults)]

use core::{future::Future, pin::Pin};

mod ext;
mod fn_service;
mod map;
mod then;

pub use self::{
    ext::SvcExt,
    fn_service::{fn_service, FnSvc},
    map::MapSvc,
    then::ThenSvc,
};

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

    #[tokio::test]
    async fn async_svc_borrowing_self() {
        struct ReturnsAsyncFn(String);

        impl Svc<()> for ReturnsAsyncFn {
            type Res = usize;
            type Fut<'fut> = impl Future<Output = Self::Res> + 'fut where Self: 'fut;

            fn exec(self: Pin<&mut Self>, _req: ()) -> Self::Fut<'_> {
                async move {
                    std::future::ready(()).await;
                    let a = self.0.as_bytes();
                    a.len()
                }
            }
        }

        let mut svc = pin!(ReturnsAsyncFn("".to_owned()));
        assert_eq!(svc.as_mut().exec(()).await, 0);
        assert_eq!(svc.exec(()).await, 0);

        let mut svc = pin!(ReturnsAsyncFn("foo".to_owned()));
        assert_eq!(svc.as_mut().exec(()).await, 3);
        assert_eq!(svc.exec(()).await, 3);
    }

    #[tokio::test]
    async fn async_svc_borrowing_req() {
        struct ReturnsAsyncFn;

        impl Svc<String> for ReturnsAsyncFn {
            type Res = usize;
            type Fut<'fut> = impl Future<Output = Self::Res> + 'fut where Self: 'fut;

            fn exec(self: Pin<&mut Self>, req: String) -> Self::Fut<'_> {
                async move {
                    std::future::ready(()).await;
                    let a = req.as_bytes();
                    a.len()
                }
            }
        }

        let mut svc = pin!(ReturnsAsyncFn);
        assert_eq!(svc.as_mut().exec("".to_owned()).await, 0);
        assert_eq!(svc.exec("foo".to_owned()).await, 3);
    }

    #[tokio::test]
    async fn async_svc_borrowing_both() {
        struct PrependStringDelay<'a>(&'a str);

        impl Svc<String> for PrependStringDelay<'_> {
            type Res = String;
            type Fut<'fut> = impl Future<Output = Self::Res> + 'fut where Self: 'fut;

            fn exec(self: Pin<&mut Self>, req: String) -> Self::Fut<'_> {
                async move {
                    std::future::ready(()).await;
                    format!("{} {req}", &self.0)
                }
            }
        }

        let prefix = "foo".to_owned();
        let mut svc = pin!(PrependStringDelay(prefix.as_str()));
        assert_eq!(svc.as_mut().exec("bar".to_owned()).await, "foo bar");
        assert_eq!(svc.exec("baz".to_owned()).await, "foo baz");
    }
}
