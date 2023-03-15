use core::{future::Future, pin::Pin};
use std::marker::PhantomData;

use crate::Svc;

pub fn fn_service<'f, F, Req, Fut, Res>(f: F) -> FnSvc<'f, F>
where
    F: FnMut(Req) -> Fut + Unpin + 'f,
    Fut: Future<Output = Res> + 'f,
{
    FnSvc {
        f,
        _phantom: PhantomData,
    }
}

pub struct FnSvc<'f, F> {
    f: F,
    _phantom: PhantomData<&'f ()>,
}

impl<F> FnSvc<'_, F> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<'f, F, Req, Fut, Res> Svc<Req> for FnSvc<'f, F>
where
    F: FnMut(Req) -> Fut + Unpin + 'f,
    Fut: Future<Output = Res> + 'f,
{
    type Res = Res;
    type Fut<'fut> = impl Future<Output = Res>
    where
        Self: 'fut;

    fn exec(mut self: Pin<&mut Self>, req: Req) -> Self::Fut<'_> {
        (self.f)(req)
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused)]

    use std::pin::pin;

    use super::*;

    #[tokio::test]
    async fn fn_service_owning() {
        let mut svc = fn_service(|n: u64| async move { n * 2 });
        let mut svc = pin!(svc);

        let res = svc.as_mut().exec(42).await;
        assert_eq!(res, 84);

        let res = svc.as_mut().exec(43).await;
        assert_eq!(res, 86);
    }

    #[tokio::test]
    async fn fn_service_borrowing() {
        let prefix = "prefix".to_owned();

        let mut svc = pin!(fn_service(|n: u64| {
            let prefix = prefix.as_str();
            return async move { format!("{prefix} {}", n * 2) };
        }));

        let res = svc.as_mut().exec(42).await;
        assert_eq!(res, "prefix 84");

        let res = svc.as_mut().exec(43).await;
        assert_eq!(res, "prefix 86");
    }
}
