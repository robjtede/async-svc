use core::{future::Future, marker::PhantomData, pin::Pin};

use pin_project_lite::pin_project;

use crate::Svc;

pin_project! {
    /// Service1(Response) -> Intermediate => Service2(Intermediate) -> Response
    pub struct ThenSvc<S1, S2, Int, Res> {
        #[pin]
        svc1: S1,

        #[pin]
        svc2: S2,

        _phantom: PhantomData<(Int, Res)>,
    }
}

impl<S1, S2, Int, Res> ThenSvc<S1, S2, Int, Res> {
    pub fn new(svc1: S1, svc2: S2) -> Self {
        Self {
            svc1,
            svc2,
            _phantom: PhantomData,
        }
    }
}

impl<S1, S2, Req, Int, Res> Svc<Req> for ThenSvc<S1, S2, Int, Res>
where
    S1: Svc<Req, Res = Int>,
    S2: Svc<Int, Res = Res>,
{
    type Res = Res;
    type Fut<'fut>
    where
        Self: 'fut,
    = impl Future<Output = Self::Res> + 'fut;

    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut<'_> {
        let this = self.project();

        async move {
            let int = this.svc1.exec(req).await;
            this.svc2.exec(int).await
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::pin_mut;

    use super::*;
    use crate::FnSvc;

    #[tokio::test]
    async fn test_then_service() {
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
