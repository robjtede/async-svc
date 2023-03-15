use core::{future::Future, marker::PhantomData, pin::Pin};

use pin_project_lite::pin_project;

use crate::Svc;

pin_project! {
    /// Service1(Response) -> Intermediate => Service2(Intermediate) -> Response
    pub struct ThenSvc<S1, S2, Req, Int, Res> {
        #[pin]
        svc1: S1,

        #[pin]
        svc2: S2,

        _phantom: PhantomData<(fn(Req) -> Res, Int,)>,
    }
}

impl<S1, S2, Req, Int, Res> ThenSvc<S1, S2, Req, Int, Res> {
    pub fn new(svc1: S1, svc2: S2) -> Self {
        Self {
            svc1,
            svc2,
            _phantom: PhantomData,
        }
    }
}

impl<S1, S2, Req, Int, Res> Svc<Req> for ThenSvc<S1, S2, Req, Int, Res>
where
    S1: Svc<Req, Res = Int>,
    S2: Svc<Int, Res = Res>,
{
    type Res = Res;
    type Fut<'fut> = impl Future<Output = Self::Res> + 'fut
    where
        Self: 'fut;

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
    use std::pin::pin;

    use super::*;
    use crate::{fn_service, FnSvc};

    async fn doubler(n: u64) -> u64 {
        n * 2
    }

    #[tokio::test]
    async fn then_owning() {
        let svc1 = FnSvc::new(doubler);
        let svc2 = FnSvc::new(doubler);
        let mut bnf = pin!(ThenSvc::new(svc1, svc2));

        assert_eq!(bnf.as_mut().exec(2).await, 8);
        assert_eq!(bnf.exec(3).await, 12);
    }

    #[tokio::test]
    async fn then_borrowing() {
        let prefix = "foo".to_owned();
        let prefix = prefix.as_str();

        let svc1 = fn_service(doubler);
        let svc2 = fn_service(|n| async move { format!("{prefix} {n}") });
        let mut bnf = pin!(ThenSvc::new(svc1, svc2));

        assert_eq!(bnf.as_mut().exec(2).await, "foo 4");
        assert_eq!(bnf.exec(3).await, "foo 6");
    }
}
