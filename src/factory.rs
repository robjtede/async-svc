use crate::Svc;

pub trait SvcFactory<Req> {
    type Cfg;
    type InitErr;
    type Svc: Svc<Req>;

    fn init_service(&self, cfg: Self::Cfg) -> Result<Self::Svc, Self::InitErr>;
}
