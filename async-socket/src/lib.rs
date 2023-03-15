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
        SF: Svc<(TcpStream, SocketAddr)> + Clone + 'static,
        SF::Fut<'static>: Send + 'static,
    {
        tracing::info!("running");

        loop {
            tracing::info!("cloning");
            let factory = factory.clone();
            tracing::info!("accepting");
            let accept = self.lst.accept().await?;

            tracing::info!("spawning factory");
            tokio::task::spawn_local(async move {
                let factory = pin!(factory);
                tracing::info!("executing factory");
                factory.exec(accept).await;
            });
        }
    }
}
