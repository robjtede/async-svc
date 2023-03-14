//! WIP async TCP socket using async-svc

#![deny(rust_2018_idioms, nonstandard_style)]
#![allow(incomplete_features)]
#![feature(type_alias_impl_trait, never_type)]

use std::{fmt, io};

use async_svc::{Svc, SvcFactory};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    pin, task,
};

pub struct Server {
    lst: TcpListener,
}

impl Server {
    pub async fn bind(addrs: impl ToSocketAddrs) -> io::Result<Self> {
        Ok(Self {
            lst: TcpListener::bind(addrs).await?,
        })
    }

    pub async fn run_using<SF>(self, factory: SF) -> io::Result<()>
    where
        SF: SvcFactory<TcpStream, Cfg = ()>,
        SF::InitErr: fmt::Debug,
        SF::InitSvc: Send + 'static,
        <SF::InitSvc as Svc<TcpStream>>::Res: Send + 'static,
        <SF::InitSvc as Svc<TcpStream>>::Fut<'static>: Send + 'static,
    {
        pin!(factory);

        loop {
            let (stream, addr) = self.lst.accept().await?;

            let svc = factory.init_svc(()).await.unwrap();

            let _ = task::spawn(async move {
                println!("connection from {}", addr);
                pin!(svc);
                svc.exec(stream).await
            });
        }
    }
}
