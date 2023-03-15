#![deny(rust_2018_idioms, nonstandard_style)]
#![allow(incomplete_features)]
#![feature(type_alias_impl_trait, never_type)]

use std::{
    future::{ready, Future, Ready},
    io,
    net::SocketAddr,
    pin::Pin,
};

use async_socket::Server;
use async_svc::Svc;
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::TcpStream,
    task::LocalSet,
    try_join,
};

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");

    let ls = LocalSet::new();

    ls.block_on(&rt, start())?;

    Ok(())
}

async fn start() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let srv = Server::bind(("127.0.0.1", 4444)).await?;

    tracing::info!("server started at {}", srv.addr());

    try_join!(srv.run_using(EchoerFactory))?;

    Ok(())
}

#[derive(Debug)]
struct EchoerFactory;

impl Svc<()> for EchoerFactory {
    type Res = Echoer;

    type Fut<'fut> = Ready<Echoer>
    where
        Self: 'fut;

    fn exec(self: Pin<&mut Self>, _req: ()) -> Self::Fut<'_> {
        ready(Echoer::default())
    }
}

#[derive(Debug, Default)]
struct Echoer {
    buffer: Vec<u8>,
}

impl Svc<(TcpStream, SocketAddr)> for Echoer {
    type Res = ();
    type Fut<'fut> = impl Future<Output = Self::Res> + Send + 'fut
    where
        Self: 'fut;

    fn exec(
        mut self: Pin<&mut Self>,
        (mut stream, _peer_addr): (TcpStream, SocketAddr),
    ) -> Self::Fut<'_> {
        async move {
            loop {
                let mut buf = [0u8; 256];

                let read = match stream.read(&mut buf).await {
                    Ok(0) => break,

                    Ok(read) => read,

                    Err(err) => {
                        tracing::error!("{err}");
                        break;
                    }
                };

                tracing::debug!("read in: {read} bytes");
                let _ = stream.write_all(&buf[..read]).await.unwrap();

                self.buffer.append(&mut buf[..read].to_vec());
            }

            tracing::debug!("whole input: {:X?}", self.buffer);
        }
    }
}
