use core::{future::Future, marker::PhantomData};

use crate::{Svc, SvcFactory};

pub struct FnFactory<Fac, Cfg> {
    fac: Fac,
    _phantom: PhantomData<Cfg>,
}

impl<Fac, Cfg, InitFut, InitErr, InitSvc, Req> SvcFactory<Req> for FnFactory<Fac, Cfg>
where
    Fac: FnMut(Cfg) -> InitFut + Clone + Unpin,
    InitFut: Future<Output = Result<InitSvc, InitErr>>,
    InitSvc: Svc<Req>,
{
    type InitSvc = InitSvc;
    type Cfg = Cfg;
    type InitErr = InitErr;
    type InitFut = InitFut;

    fn init_svc(&self, cfg: Self::Cfg) -> Self::InitFut {
        (self.fac.clone())(cfg)
    }
}

pub fn fn_factory<Fac, Cfg, InitFut, InitErr, OutSvc, Req>(fac: Fac) -> FnFactory<Fac, Cfg>
where
    Fac: FnMut(Cfg) -> InitFut + Unpin,
    InitFut: Future<Output = Result<OutSvc, InitErr>>,
    OutSvc: Svc<Req>,
{
    FnFactory {
        fac,
        _phantom: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use futures_util::{future::ok, pin_mut};

    use super::*;
    use crate::fn_service;

    #[tokio::test]
    async fn test_fn_factory_no_cfg() {
        let no_cfg_fac = fn_factory(|_: ()| ok::<_, ()>(fn_service(|n: u64| async move { n * 2 })));
        pin_mut!(no_cfg_fac);

        {
            let svc = no_cfg_fac.init_svc(()).await.unwrap();
            pin_mut!(svc);
            let res = svc.exec(42).await;
            assert_eq!(res, 84);
        }

        {
            let svc = no_cfg_fac.init_svc(()).await.unwrap();
            pin_mut!(svc);
            let res = svc.exec(43).await;
            assert_eq!(res, 86);
        }
    }

    #[tokio::test]
    async fn test_fn_factory_with_cfg() {
        let cfg_fac = fn_factory(|shift: u64| {
            ok::<_, ()>(fn_service(move |n: u64| async move { (n * 2) + shift }))
        });
        pin_mut!(cfg_fac);

        {
            let svc = cfg_fac.init_svc(7).await.unwrap();
            pin_mut!(svc);
            let res = svc.exec(42).await;
            assert_eq!(res, 91);
        }

        {
            let svc = cfg_fac.init_svc(7).await.unwrap();
            pin_mut!(svc);
            let res = svc.exec(43).await;
            assert_eq!(res, 93);
        }
    }
}
