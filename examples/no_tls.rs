#[macro_use]
extern crate capnp_rpc;

pub mod echo_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema/echo_capnp.rs"));
}

pub mod server {
    use crate::echo_capnp::echo;
    use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
    use futures::{AsyncReadExt, FutureExt, TryFutureExt};

    struct Echo;

    impl echo::Server for Echo {
        fn echo(
            &mut self,
            params: echo::EchoParams,
            mut results: echo::EchoResults,
        ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
            let input = pry!(pry!(params.get()).get_input());
            results.get().set_output(input);
            ::capnp::capability::Promise::ok(())
        }
    }

    pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
        use std::net::ToSocketAddrs;
        let args: Vec<String> = ::std::env::args().collect();
        if args.len() != 3 {
            println!("usage: {} server HOST:PORT", args[0]);
            return Ok(());
        }

        let addr = args[2]
            .to_socket_addrs()
            .unwrap()
            .next()
            .expect("could not parse address");

        tokio::task::LocalSet::new()
            .run_until(async move {
                let mut socket = tokio::net::TcpListener::bind(&addr).await?;

                let echo_server: echo::Client = capnp_rpc::new_client(Echo);

                loop {
                    let (stream, _addr) = socket.accept().await?;
                    stream.set_nodelay(true)?;
                    let (reader, writer) =
                        tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();

                    let network = twoparty::VatNetwork::new(
                        reader,
                        writer,
                        rpc_twoparty_capnp::Side::Server,
                        Default::default(),
                    );

                    let rpc_system =
                        RpcSystem::new(Box::new(network), Some(echo_server.clone().client));
                    tokio::task::spawn_local(Box::pin(
                        rpc_system
                            .map_err(|e| println!("error: {:?}", e))
                            .map(|_| ()),
                    ));
                }
            })
            .await
    }
}

pub mod client {
    use crate::echo_capnp::echo;
    use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
    use futures::{AsyncReadExt, FutureExt};

    async fn try_main(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        use std::net::ToSocketAddrs;

        let addr = args[2]
            .to_socket_addrs()?
            .next()
            .expect("could not parse address");

        let stream = tokio::net::TcpStream::connect(&addr).await?;
        stream.set_nodelay(true)?;
        let (reader, writer) =
            tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();

        let network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        ));
        let mut rpc_system = RpcSystem::new(network, None);
        let echo_client: echo::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

        tokio::task::spawn_local(Box::pin(rpc_system.map(|_| ())));

        let mut request = echo_client.echo_request();
        request.get().set_input("hello");
        let response = request.send().promise.await?;
        let output = response.get()?.get_output().unwrap();
        println!("{}", output);
        Ok(())
    }

    pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<String> = ::std::env::args().collect();
        if args.len() != 3 {
            println!("usage: {} client HOST:PORT", args[0]);
            return Ok(());
        }
        tokio::task::LocalSet::new().run_until(try_main(args)).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() >= 2 {
        match &args[1][..] {
            "client" => return client::main().await,
            "server" => return server::main().await,
            _ => (),
        }
    }

    println!("usage: {} [client | server] HOST:PORT", args[0]);
    Ok(())
}
