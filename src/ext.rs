use std::marker::Sized;

use crate::{MapSvc, Svc, ThenSvc};

pub trait SvcExt<Req>: Svc<Req> {
    fn map<F, Res>(self, f: F) -> MapSvc<Self, F, Res>
    where
        Self: Sized,
        F: FnMut(Self::Res) -> Res,
    {
        MapSvc::new(self, f)
    }
    
    fn then<S2, Int, Res>(self, svc2: S2) -> ThenSvc<Self, S2, Int, Res>
    where
        Self: Sized,
        S2: Svc<Self::Res, Res = Res>,
    {
        ThenSvc::new(self, svc2)
    }
}
