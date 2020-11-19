use std::convert::Infallible;

use crate::Svc;

pub trait SvcFactory<Req> {
    type Cfg;
    type InitErr;
    type Svc: Svc<Req>;

    fn init_svc(&self, cfg: Self::Cfg) -> Result<Self::Svc, Self::InitErr>;
}

impl<S, Req> SvcFactory<Req> for S
where
    S: Svc<Req> + Clone,
{
    type Cfg = ();
    type InitErr = Infallible;
    type Svc = S;

    fn init_svc(&self, _: Self::Cfg) -> Result<Self::Svc, Self::InitErr> {
        Ok(self.clone())
    }
}
