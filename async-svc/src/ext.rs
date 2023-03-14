use core::marker::Sized;

// use crate::{box_svc, boxed::BoxSvc, MapSvc, Svc, ThenSvc};
use crate::{MapSvc, Svc, ThenSvc};

pub trait SvcExt<Req>: Svc<Req> {
    fn map<F, Res>(self, f: F) -> MapSvc<Self, F, Res>
    where
        Self: Sized,
        F: FnMut(Self::Res) -> Res,
    {
        MapSvc::new(self, f)
    }

    fn then<S2, Res>(self, svc2: S2) -> ThenSvc<Self, S2, Self::Res, Res>
    where
        Self: Sized,
        S2: Svc<Self::Res, Res = Res>,
    {
        ThenSvc::new(self, svc2)
    }

    // fn boxed(self) -> BoxSvc<Req, Self::Res>
    // where
    //     Self: Sized + 'static,
    //     Req: 'static,
    // {
    //     box_svc(self)
    // }
}
