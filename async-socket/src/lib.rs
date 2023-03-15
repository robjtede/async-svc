//! WIP async TCP socket using async-svc

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms, nonstandard_style)]
#![feature(type_alias_impl_trait, never_type)]

use std::{io, net::SocketAddr, pin::pin};

use async_svc::Svc;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

pub struct Server {
    pub lst: TcpListener,
}

impl Server {
    pub async fn bind(addrs: impl ToSocketAddrs) -> io::Result<Self> {
        Ok(Self {
            lst: TcpListener::bind(addrs).await?,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.lst.local_addr().unwrap()
    }

    pub async fn run_using<SF>(self, factory: SF) -> io::Result<()>
    where
        SF: Svc<()>,
        SF::Res: Svc<(TcpStream, SocketAddr)> + 'static,
        <SF::Res as Svc<(TcpStream, SocketAddr)>>::Fut<'static>: 'static,
    {
        let mut factory = pin!(factory);

        loop {
            let svc = factory.as_mut().exec(()).await;
            let accept = self.lst.accept().await?;

            tokio::task::spawn_local(async move {
                let svc = pin!(svc);
                svc.exec(accept).await;
            });
        }
    }
}
