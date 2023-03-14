use core::{future::Future, ops::Deref};

use crate::Svc;

pub trait SvcFactory<Req> {
    type InitSvc: Svc<Req>;
    type Cfg;
    type InitErr;
    type InitFut: Future<Output = Result<Self::InitSvc, Self::InitErr>>;

    fn init_svc(&self, cfg: Self::Cfg) -> Self::InitFut;
}

/// Covers all manner of smart pointers. (Box, Rc, Arc, etc.)
impl<P, SF, Req> SvcFactory<Req> for P
where
    P: Deref<Target = SF>,
    SF: SvcFactory<Req>,
{
    type InitSvc = SF::InitSvc;
    type Cfg = SF::Cfg;
    type InitErr = SF::InitErr;
    type InitFut = SF::InitFut;

    fn init_svc(&self, cfg: SF::Cfg) -> SF::InitFut {
        self.deref().init_svc(cfg)
    }
}

#[cfg(test)]
mod test_factory {
    use alloc::rc::Rc;

    use futures_util::future::ok;

    // use super::*;
    use crate::{fn_factory, fn_service};

    #[tokio::test]
    async fn test_smart_pointer_factories() {
        let doubler_fac =
            fn_factory(|_: ()| ok::<_, ()>(fn_service(|n: u64| async move { n * 2 })));
        let fac = Rc::pin(doubler_fac);
        fac.init_svc(()).await.unwrap();
    }
}
