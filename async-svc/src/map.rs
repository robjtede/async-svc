use core::{future::Future, marker::PhantomData, pin::Pin};

use pin_project_lite::pin_project;

use crate::Svc;

pin_project! {
    pub struct MapSvc<S, F, Res> {
        #[pin]
        svc: S,
        mapper: F,
        _res: PhantomData<Res>,
    }
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
    type Fut<'fut>
    where
        Self: 'fut,
    = impl Future<Output = Self::Res>;

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
