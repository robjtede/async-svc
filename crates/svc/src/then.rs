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
    type Permit = (S1::Permit, S2::Permit);
    type AcquireFut<'fut>
        = impl Future<Output = Option<Self::Permit>> + 'fut
    where
        Self: 'fut;
    type Res = Res;
    type Fut<'fut>
        = impl Future<Output = Self::Res> + 'fut
    where
        Self: 'fut;

    fn acquire(&self) -> Self::AcquireFut<'_> {
        async move {
            let permit1 = self.svc1.acquire().await?;
            let permit2 = self.svc2.acquire().await?;
            Some((permit1, permit2))
        }
    }

    fn exec(self: Pin<&mut Self>, (permit1, permit2): Self::Permit, req: Req) -> Self::Fut<'_> {
        let this = self.project();

        async move {
            let int = this.svc1.exec(permit1, req).await;
            this.svc2.exec(permit2, int).await
        }
    }
}

#[cfg(test)]
mod tests {
    use core::{
        cell::Cell,
        future::{ready, Ready},
    };
    use std::{pin::pin, rc::Rc};

    use super::*;
    use crate::{fn_service, oneshot, FnSvc};

    async fn doubler(n: u64) -> u64 {
        n * 2
    }

    #[tokio::test]
    async fn acquire_releases_first_permit_when_second_service_is_closed() {
        struct DropPermit(Rc<Cell<usize>>);

        impl Drop for DropPermit {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        struct First(Rc<Cell<usize>>);

        impl Svc<()> for First {
            type Permit = DropPermit;
            type AcquireFut<'fut>
                = Ready<Option<Self::Permit>>
            where
                Self: 'fut;
            type Res = ();
            type Fut<'fut>
                = Ready<Self::Res>
            where
                Self: 'fut;

            fn acquire(&self) -> Self::AcquireFut<'_> {
                ready(Some(DropPermit(Rc::clone(&self.0))))
            }

            fn exec(self: Pin<&mut Self>, _permit: Self::Permit, _req: ()) -> Self::Fut<'_> {
                ready(())
            }
        }

        struct Closed;

        impl Svc<()> for Closed {
            type Permit = ();
            type AcquireFut<'fut>
                = Ready<Option<Self::Permit>>
            where
                Self: 'fut;
            type Res = ();
            type Fut<'fut>
                = Ready<Self::Res>
            where
                Self: 'fut;

            fn acquire(&self) -> Self::AcquireFut<'_> {
                ready(None)
            }

            fn exec(self: Pin<&mut Self>, (): Self::Permit, _req: ()) -> Self::Fut<'_> {
                ready(())
            }
        }

        let drops = Rc::new(Cell::new(0));
        let svc = ThenSvc::<_, _, (), (), ()>::new(First(Rc::clone(&drops)), Closed);

        assert!(svc.acquire().await.is_none());
        assert_eq!(drops.get(), 1);
    }

    #[tokio::test]
    async fn then_owning() {
        let svc1 = FnSvc::new(doubler);
        let svc2 = FnSvc::new(doubler);
        let mut bnf = pin!(ThenSvc::new(svc1, svc2));

        assert_eq!(oneshot(bnf.as_mut(), 2).await, 8);
        assert_eq!(oneshot(bnf, 3).await, 12);
    }

    #[tokio::test]
    async fn then_borrowing() {
        let prefix = "foo".to_owned();
        let prefix = prefix.as_str();

        let svc1 = fn_service(doubler);
        let svc2 = fn_service(|n| async move { format!("{prefix} {n}") });
        let mut bnf = pin!(ThenSvc::new(svc1, svc2));

        assert_eq!(oneshot(bnf.as_mut(), 2).await, "foo 4");
        assert_eq!(oneshot(bnf, 3).await, "foo 6");
    }
}
