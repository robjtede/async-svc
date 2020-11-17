use std::{future::Future, pin::Pin};

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
    type Fut = Fut;

    fn exec(mut self: Pin<&mut Self>, req: Req) -> Self::Fut {
        (self.f)(req)
    }
}

#[cfg(test)]
mod tests {
    use futures_util::pin_mut;

    use super::*;

    #[tokio::test]
    async fn test_fn_service() {
        async fn doubler(n: u64) -> u64 {
            n * 2
        }

        let svc = FnSvc::new(doubler);
        pin_mut!(svc);

        let res = svc.exec(42).await;
        assert_eq!(res, 84);
    }
}
