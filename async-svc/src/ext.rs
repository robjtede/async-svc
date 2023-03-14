use core::marker::Sized;

use crate::{MapSvc, Svc, ThenSvc};

pub trait SvcExt<Req>: Svc<Req> {
    fn map<'a, F, Res>(self, mapper: F) -> MapSvc<'a, Self, F, Req, Res>
    where
        Self: Sized,
        F: FnMut(Self::Res) -> Res,
    {
        MapSvc::new(self, mapper)
    }

    fn then<S2, Res>(self, svc2: S2) -> ThenSvc<Self, S2, Req, Self::Res, Res>
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

impl<S, Req> SvcExt<Req> for S where S: Svc<Req> {}

#[cfg(test)]
mod tests {
    use core::fmt;
    use std::pin::pin;

    use crate::fn_service;

    use super::*;

    async fn doubler(n: u64) -> u64 {
        n * 2
    }

    fn prefixer(val: impl fmt::Display) -> String {
        format!("foo {val}")
    }

    #[tokio::test]
    async fn map() {
        let mut svc = pin!(fn_service(doubler).map(prefixer));
        assert_eq!(svc.as_mut().exec(1).await, "foo 2");
        assert_eq!(svc.exec(3).await, "foo 6");
    }

    #[tokio::test]
    async fn then() {
        let mut svc = pin!(fn_service(doubler).then(fn_service(doubler)));
        assert_eq!(svc.as_mut().exec(1).await, 4);
        assert_eq!(svc.exec(3).await, 4);
    }
}
