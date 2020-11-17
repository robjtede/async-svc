use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use pin_project::pin_project;

use crate::Svc;

#[pin_project]
pub struct MapSvc<S, F, Res> {
    #[pin]
    svc: S,
    mapper: F,
    _res: PhantomData<Res>,
}

impl<S, F, Res> MapSvc<S, F, Res> {
    pub fn new(svc: S, mapper: F) -> Self {
        Self {
            svc,
            mapper,
            _res: PhantomData,
        }
    }
}

impl<S, Req, F, Res> Svc<Req> for MapSvc<S, F, Res>
where
    S: Svc<Req>,
    F: FnMut(S::Res) -> Res + Clone,
{
    type Res = Res;
    type Fut = MapSvcFut<S, F, Req, Res>;

    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut {
        let mapper = self.mapper.clone();
        let this = self.project();
        MapSvcFut::new(this.svc.exec(req), mapper)
    }
}

#[pin_project]
pub struct MapSvcFut<S, F, Req, Res>
where
    S: Svc<Req>,
    F: FnMut(S::Res) -> Res,
{
    mapper: F,
    #[pin]
    fut: S::Fut,
}

impl<S, F, Req, Res> MapSvcFut<S, F, Req, Res>
where
    S: Svc<Req>,
    F: FnMut(S::Res) -> Res,
{
    pub fn new(fut: S::Fut, mapper: F) -> Self {
        Self { fut, mapper }
    }
}

impl<S, F, Req, Res> Future for MapSvcFut<S, F, Req, Res>
where
    S: Svc<Req>,
    F: FnMut(S::Res) -> Res,
{
    type Output = Res;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let res = ready!(this.fut.poll(cx));
        Poll::Ready((this.mapper)(res))
    }
}

#[cfg(test)]
mod tests {
    use futures_util::pin_mut;

    use super::*;
    use crate::FnSvc;

    #[tokio::test]
    async fn test_map_service() {
        async fn doubler(n: u64) -> u64 {
            n * 2
        }

        let svc = FnSvc::new(doubler);
        let bnf = MapSvc::new(svc, |res| res + 2);

        pin_mut!(bnf);

        let res = bnf.exec(42).await;
        assert_eq!(res, 86);
    }
}
