use alloc::boxed::Box;
use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::Svc;

pub type BoxFut<Res> = Pin<Box<dyn Future<Output = Res>>>;
pub type BoxSvc<Req, Res> = Pin<Box<dyn Svc<Req, Res = Res, Fut = BoxFut<Res>>>>;

impl<Req, Res> Svc<Req> for BoxSvc<Req, Res> {
    type Res = Res;
    type Fut<'fut> = impl Future<Output = Self::Res>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: Context<'_>) -> Poll<()> {
        self.as_mut().poll_ready(cx)
    }

    fn exec(mut self: Pin<&mut Self>, req: Req) -> Self::Fut<'_> {
        self.as_mut().exec(req)
    }
}

pub fn box_svc<S, Req>(svc: S) -> BoxSvc<Req, S::Res>
where
    S: Svc<Req>,
{
    Box::pin(SvcWrapper::new(svc))
}

#[pin_project]
struct SvcWrapper<S: Svc<Req>, Req> {
    #[pin]
    svc: S,
    _req: PhantomData<Req>,
}

impl<S: Svc<Req>, Req> SvcWrapper<S, Req> {
    fn new(svc: S) -> Self {
        Self {
            svc,
            _req: PhantomData,
        }
    }
}

impl<S, Req> Svc<Req> for SvcWrapper<S, Req>
where
    S: Svc<Req>,
{
    type Res = S::Res;
    type Fut<'fut> = impl Future<Output = Self::Res>;

    fn poll_ready(self: Pin<&mut Self>, cx: Context<'_>) -> Poll<()> {
        let this = self.project();
        this.svc.poll_ready(cx)
    }

    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut<'_> {
        let this = self.project();
        Box::pin(this.svc.exec(req))
    }
}

#[cfg(test)]
mod tests {
    use alloc::rc::Rc;
    use core::{cell::RefCell};

    use super::*;
    use crate::FnSvc;

    #[tokio::test]
    async fn test_boxed() {
        let sum = Rc::new(RefCell::new(0));
        let sum2 = Rc::clone(&sum);
        let running_sum = move |n: u64| {
            let sum = Rc::clone(&sum2);
            Box::pin(async move {
                *sum.borrow_mut() += n;
                *sum.borrow()
            })
        };

        let svc = FnSvc::new(running_sum);
        let mut boxed_srv = box_svc(svc);

        let res = boxed_srv.as_mut().exec(20).await;
        assert_eq!(res, 20);

        let res = boxed_srv.as_mut().exec(14).await;
        assert_eq!(res, 34);
    }
}
