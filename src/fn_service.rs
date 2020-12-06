use core::{future::Future, pin::Pin};

use crate::Svc;

pub struct FnSvc<F> {
    f: F,
}

impl<F> FnSvc<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F, Req, Fut, Res> Svc<Req> for FnSvc<F>
where
    F: FnMut(Req) -> Fut + Unpin,
    Fut: Future<Output = Res>,
{
    type Res = Res;
    type Fut<'fut> = impl Future<Output = Res>;

    fn exec(mut self: Pin<&mut Self>, req: Req) -> Self::Fut<'_> {
        (self.f)(req)
    }
}

pub fn fn_service<F, Req, Fut, Res>(f: F) -> FnSvc<F>
where
    F: FnMut(Req) -> Fut + Unpin,
    Fut: Future<Output = Res>,
{
    FnSvc { f }
}

#[cfg(test)]
mod tests {
    use futures_util::pin_mut;

    use super::*;

    #[tokio::test]
    async fn test_fn_service() {
        let svc = fn_service(|n: u64| async move { n * 2 });
        pin_mut!(svc);

        let res = svc.as_mut().exec(42).await;
        assert_eq!(res, 84);

        let res = svc.exec(43).await;
        assert_eq!(res, 86);
    }
}
