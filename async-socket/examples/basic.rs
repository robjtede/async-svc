#![deny(rust_2018_idioms, nonstandard_style)]
#![allow(incomplete_features)]
#![feature(generic_associated_types, type_alias_impl_trait, never_type)]

use std::{
    convert::Infallible,
    future::{ready, Future},
    io,
    pin::Pin,
};

use async_socket::Server;
use async_svc::{Svc, SvcFactory};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let srv = Server::bind(("127.0.0.1", 4444)).await?;
    srv.run_using::<EchoerFactory>(EchoerFactory).await
}

#[derive(Debug, Default)]
struct EchoerFactory;

impl SvcFactory<TcpStream> for EchoerFactory {
    type InitSvc = Echoer;
    type Cfg = ();
    type InitErr = Infallible;
    type InitFut = impl Future<Output = Result<Self::InitSvc, Self::InitErr>>;

    fn init_svc(&self, _cfg: Self::Cfg) -> Self::InitFut {
        ready(Ok(Echoer::default()))
    }
}

#[derive(Debug, Default)]
struct Echoer {
    buffer: Vec<u8>,
}

impl Svc<TcpStream> for Echoer {
    type Res = ();
    type Fut<'fut> = impl Future<Output = Self::Res> + Send;

    fn exec(mut self: Pin<&mut Self>, mut stream: TcpStream) -> Self::Fut<'_> {
        println!("exec-ing");

        async move {
            println!("async-ing");

            loop {
                let mut buf = [0u8; 128];
                let read = match stream.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(read) => read,

                    Err(err) => {
                        eprintln!("{}", err);
                        break;
                    }
                };
                println!("read in: {} bytes", read);
                let _ = stream.write_all(&buf[..read]).await.unwrap();

                self.buffer.append(&mut buf[..read].to_vec());
            }

            println!("whole input: {:?}", self.buffer);
        }
    }
}
