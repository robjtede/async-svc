//! WIP Modern Service Trait

#![feature(impl_trait_in_assoc_type)]

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
    /// Owned permission to process one request.
    type Permit;

    /// Permit acquisition future.
    type AcquireFut<'fut>: Future<Output = Option<Self::Permit>> + 'fut
    where
        Self: 'fut;

    /// Output type.
    type Res;

    /// Response future.
    type Fut<'fut>: Future<Output = Self::Res> + 'fut
    where
        Self: 'fut;

    /// Acquires permission to process one request.
    ///
    /// A pending future applies backpressure. `None` means that the service is
    /// permanently closed and cannot process another request.
    fn acquire(&self) -> Self::AcquireFut<'_>;

    /// Processes request, producing a future that outputs the response type.
    fn exec(self: Pin<&mut Self>, permit: Self::Permit, req: Req) -> Self::Fut<'_>;
}

impl<S, Req> Svc<Req> for Pin<Box<S>>
where
    S: Svc<Req>,
{
    type Permit = S::Permit;
    type AcquireFut<'fut>
        = S::AcquireFut<'fut>
    where
        Self: 'fut;
    type Res = S::Res;
    type Fut<'fut>
        = S::Fut<'fut>
    where
        Self: 'fut;

    fn acquire(&self) -> Self::AcquireFut<'_> {
        self.as_ref().get_ref().acquire()
    }

    fn exec(self: Pin<&mut Self>, permit: Self::Permit, req: Req) -> Self::Fut<'_> {
        self.get_mut().as_mut().exec(permit, req)
    }
}

#[cfg(test)]
async fn oneshot<S, Req>(mut svc: Pin<&mut S>, req: Req) -> S::Res
where
    S: Svc<Req>,
{
    let permit = svc.as_ref().get_ref().acquire().await.unwrap();
    svc.as_mut().exec(permit, req).await
}

#[cfg(test)]
mod tests {
    use core::future::{ready, Ready};
    use std::pin::pin;

    use super::*;

    #[tokio::test]
    async fn acquire_returns_owned_permit() {
        struct OwnedPermit(String);
        struct PermitSvc;

        impl Svc<String> for PermitSvc {
            type Permit = OwnedPermit;
            type AcquireFut<'fut>
                = Ready<Option<Self::Permit>>
            where
                Self: 'fut;
            type Res = String;
            type Fut<'fut>
                = Ready<Self::Res>
            where
                Self: 'fut;

            fn acquire(&self) -> Self::AcquireFut<'_> {
                ready(Some(OwnedPermit("permit".to_owned())))
            }

            fn exec(self: Pin<&mut Self>, permit: Self::Permit, req: String) -> Self::Fut<'_> {
                ready(format!("{} {req}", permit.0))
            }
        }

        let mut svc = pin!(PermitSvc);
        let permit = svc.as_ref().get_ref().acquire().await.unwrap();
        assert_eq!(
            svc.as_mut().exec(permit, "used".to_owned()).await,
            "permit used"
        );
    }

    #[tokio::test]
    async fn acquire_returns_none_when_closed() {
        struct Closed;

        impl Svc<()> for Closed {
            type Permit = ();
            type AcquireFut<'fut>
                = Ready<Option<Self::Permit>>
            where
                Self: 'fut;
            type Res = ();
            type Fut<'fut>
                = Ready<Self::Res>
            where
                Self: 'fut;

            fn acquire(&self) -> Self::AcquireFut<'_> {
                ready(None)
            }

            fn exec(self: Pin<&mut Self>, (): Self::Permit, _req: ()) -> Self::Fut<'_> {
                ready(())
            }
        }

        assert!(Closed.acquire().await.is_none());
    }

    #[tokio::test]
    async fn async_svc_borrowing_self() {
        struct ReturnsAsyncFn(String);

        impl Svc<()> for ReturnsAsyncFn {
            type Permit = ();
            type AcquireFut<'fut>
                = core::future::Ready<Option<Self::Permit>>
            where
                Self: 'fut;
            type Res = usize;
            type Fut<'fut>
                = impl Future<Output = Self::Res> + 'fut
            where
                Self: 'fut;

            fn acquire(&self) -> Self::AcquireFut<'_> {
                core::future::ready(Some(()))
            }

            fn exec(self: Pin<&mut Self>, (): Self::Permit, _req: ()) -> Self::Fut<'_> {
                async move {
                    std::future::ready(()).await;
                    let a = self.0.as_bytes();
                    a.len()
                }
            }
        }

        let mut svc = pin!(ReturnsAsyncFn("".to_owned()));
        assert_eq!(oneshot(svc.as_mut(), ()).await, 0);
        assert_eq!(oneshot(svc, ()).await, 0);

        let mut svc = pin!(ReturnsAsyncFn("foo".to_owned()));
        assert_eq!(oneshot(svc.as_mut(), ()).await, 3);
        assert_eq!(oneshot(svc, ()).await, 3);
    }

    #[tokio::test]
    async fn async_svc_borrowing_req() {
        struct ReturnsAsyncFn;

        impl Svc<String> for ReturnsAsyncFn {
            type Permit = ();
            type AcquireFut<'fut>
                = core::future::Ready<Option<Self::Permit>>
            where
                Self: 'fut;
            type Res = usize;
            type Fut<'fut>
                = impl Future<Output = Self::Res> + 'fut
            where
                Self: 'fut;

            fn acquire(&self) -> Self::AcquireFut<'_> {
                core::future::ready(Some(()))
            }

            fn exec(self: Pin<&mut Self>, (): Self::Permit, req: String) -> Self::Fut<'_> {
                async move {
                    std::future::ready(()).await;
                    let a = req.as_bytes();
                    a.len()
                }
            }
        }

        let mut svc = pin!(ReturnsAsyncFn);
        assert_eq!(oneshot(svc.as_mut(), "".to_owned()).await, 0);
        assert_eq!(oneshot(svc, "foo".to_owned()).await, 3);
    }

    #[tokio::test]
    async fn async_svc_borrowing_both() {
        struct PrependStringDelay<'a>(&'a str);

        impl Svc<String> for PrependStringDelay<'_> {
            type Permit = ();
            type AcquireFut<'fut>
                = core::future::Ready<Option<Self::Permit>>
            where
                Self: 'fut;
            type Res = String;
            type Fut<'fut>
                = impl Future<Output = Self::Res> + 'fut
            where
                Self: 'fut;

            fn acquire(&self) -> Self::AcquireFut<'_> {
                core::future::ready(Some(()))
            }

            fn exec(self: Pin<&mut Self>, (): Self::Permit, req: String) -> Self::Fut<'_> {
                async move {
                    std::future::ready(()).await;
                    format!("{} {req}", self.0)
                }
            }
        }

        let prefix = "foo".to_owned();
        let mut svc = pin!(PrependStringDelay(prefix.as_str()));
        assert_eq!(oneshot(svc.as_mut(), "bar".to_owned()).await, "foo bar");
        assert_eq!(oneshot(svc, "baz".to_owned()).await, "foo baz");
    }
}
