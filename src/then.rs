use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use pin_project::pin_project;

use crate::Svc;

/// Service1(Response) -> Intermediate => Service2(Intermediate) -> Response
#[pin_project]
pub struct ThenSvc<S1, S2, Int, Res> {
    #[pin]
    svc1: S1,
    svc2: Option<S2>,
    _int: PhantomData<Int>,
    _res: PhantomData<Res>,
}

impl<S1, S2, Int, Res> ThenSvc<S1, S2, Int, Res> {
    pub fn new(svc1: S1, svc2: S2) -> Self {
        Self {
            svc1,
            svc2: Some(svc2),
            _int: PhantomData,
            _res: PhantomData,
        }
    }
}

impl<S1, S2, Req, Int, Res> Svc<Req> for ThenSvc<S1, S2, Int, Res>
where
    S1: Svc<Req, Res = Int>,
    S2: Svc<Int, Res = Res>,
{
    type Res = Res;
    type Fut = ThenSvcFut<S1, S2, Req, Int, Res>;

    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut {
        let this = self.project();
        ThenSvcFut {
            state: State::Svc1(
                this.svc1.exec(req),
                this.svc2.take().expect("Service must not be executed twice."),
            ),
        }
    }
}

#[pin_project]
pub struct ThenSvcFut<S1, S2, Req, Int, Res>
where
    S1: Svc<Req, Res = Int>,
    S2: Svc<Int, Res = Res>,
{
    #[pin]
    state: State<S1, S2, Req, Int, Res>,
}

#[pin_project(project = StateProj)]
enum State<S1, S2, Req, Int, Res>
where
    S1: Svc<Req, Res = Int>,
    S2: Svc<Int, Res = Res>,
{
    Svc1(#[pin] S1::Fut, #[pin] S2),
    Svc2(#[pin] S2::Fut),
    Done,
}

impl<S1, S2, Req, Int, Res> Future for ThenSvcFut<S1, S2, Req, Int, Res>
where
    S1: Svc<Req, Res = Int>,
    S2: Svc<Int, Res = Res>,
{
    type Output = Res;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        match this.state.as_mut().project() {
            StateProj::Svc1(s1_fut, s2) => {
                let s1_res = ready!(s1_fut.poll(cx));
                let s2_exec = s2.exec(s1_res);
                this.state.set(State::Svc2(s2_exec));
                self.poll(cx)
            }
            StateProj::Svc2(s2_fut) => {
                let s2_res = ready!(s2_fut.poll(cx));
                this.state.set(State::Done);
                Poll::Ready(s2_res)
            }
            StateProj::Done => panic!("Future must not be polled after it returned `Poll::Ready`."),
        }
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

        let svc1 = FnSvc::new(doubler);
        let svc2 = FnSvc::new(doubler);
        let bnf = ThenSvc::new(svc1, svc2);

        pin_mut!(bnf);

        let res = bnf.exec(42).await;
        assert_eq!(res, 168);
    }
}
