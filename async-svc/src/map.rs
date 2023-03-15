use core::{future::Future, marker::PhantomData, pin::Pin};

use pin_project_lite::pin_project;

use crate::Svc;

pin_project! {
    pub struct MapSvc<'f, S, F, Req, Res> {
        #[pin]
        svc: S,
        mapper: F,
        _phantom: PhantomData<(&'f (), fn(Req) -> Res,)>,
    }
}

impl<S, F, Req, Res> MapSvc<'_, S, F, Req, Res> {
    pub fn new(svc: S, mapper: F) -> Self {
        Self {
            svc,
            mapper,
            _phantom: PhantomData,
        }
    }
}

impl<'f, S, Req, F, Res> Svc<Req> for MapSvc<'f, S, F, Req, Res>
where
    S: Svc<Req>,
    F: FnMut(S::Res) -> Res + 'f,
{
    type Res = Res;
    type Fut<'fut> = impl Future<Output = Self::Res> + 'fut
    where
        Self: 'fut;

    fn exec(self: Pin<&mut Self>, req: Req) -> Self::Fut<'_> {
        let this = self.project();

        async move {
            let res = this.svc.exec(req).await;
            (this.mapper)(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;

    use super::*;
    use crate::fn_service;

    async fn doubler(n: u64) -> u64 {
        n * 2
    }

    #[tokio::test]
    async fn map_owning() {
        let svc = fn_service(doubler);
        let mut bnf = pin!(MapSvc::new(svc, |res| res + 2));

        assert_eq!(bnf.as_mut().exec(42).await, 86);
        assert_eq!(bnf.exec(2).await, 6);
    }

    #[tokio::test]
    async fn map_borrowing() {
        let prefix = "foo".to_owned();
        let prefix = prefix.as_str();

        let svc = fn_service(doubler);
        let mut bnf = pin!(MapSvc::new(svc, |res| format!("{prefix} {res}")));

        assert_eq!(bnf.as_mut().exec(42).await, "foo 84");
        assert_eq!(bnf.exec(2).await, "foo 4");
    }
}
