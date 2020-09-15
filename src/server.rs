use std::sync::Arc;

use crate::echo_capnp::echo;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::{AsyncReadExt, FutureExt, TryFutureExt};

use rustls::ServerConfig;
//use rustls::AllowAnyAuthenticatedClient;
use rcgen::generate_simple_self_signed;
use rustls::Certificate;
use rustls::NoClientAuth;
use rustls::PrivateKey;
use tokio_rustls::TlsAcceptor;

struct Echo {
    email: String,
}

impl echo::Server for Echo {
    fn echo(
        &mut self,
        params: echo::EchoParams,
        mut results: echo::EchoResults,
    ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
        let input = pry!(pry!(params.get()).get_input());
        results
            .get()
            .set_output(&format!("{}:{}", self.email, input));
        ::capnp::capability::Promise::ok(())
    }
}

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} server HOST:PORT", args[0]);
        return Ok(());
    }
    tokio::task::LocalSet::new()
        .run_until(try_main(args[2].clone()))
        .await
}

pub async fn try_main(addr_port: String) -> Result<(), Box<dyn std::error::Error>> {
    use std::net::ToSocketAddrs;

    let addr = addr_port
        .to_socket_addrs()?
        .next()
        .expect("could not parse address");

    let subject_alt_names = vec!["localhost".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    // The certificate is now valid for localhost and the domain "hello.world.example"
    println!("{}", cert.serialize_pem().unwrap());
    println!("{}", cert.serialize_private_key_pem());

    let pcert = Certificate(cert.serialize_der().unwrap());
    let pkey = PrivateKey(cert.serialize_private_key_der());

    /*
    let mut client_auth_roots = RootCertStore::empty();
    let roots = ::load_certs("test-ca/rsa/end.fullchain");
    for root in &roots {
        client_auth_roots.add(&root).unwrap();
    }
    */
    let client_auth = NoClientAuth::new();
    //let client_auth = AllowAnyAuthenticatedClient::new(client_auth_roots);

    let mut config = ServerConfig::new(client_auth);
    //config.set_single_cert(roots, ::load_private_key("test-ca/rsa/end.key")).unwrap();
    config
        .set_single_cert(vec![pcert], pkey)
        .expect("invalid key or certificate");
    let acceptor = TlsAcceptor::from(Arc::new(config));

    let mut listener = tokio::net::TcpListener::bind(&addr).await?;

    loop {
        let (stream, _addr) = listener.accept().await?;
        stream.set_nodelay(true)?;
        let acceptor = acceptor.clone();
        let stream = acceptor.accept(stream).await?;
        let (reader, writer) =
            tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();

        let echo = Echo {
            email: "my@email.com".to_string(),
        };
        let echo_server: echo::Client = capnp_rpc::new_client(echo);
        let network = twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Server,
            Default::default(),
        );
        let rpc_system = RpcSystem::new(Box::new(network), Some(echo_server.client));
        tokio::task::spawn_local(Box::pin(
            rpc_system
                .map_err(|e| println!("error: {:?}", e))
                .map(|_| ()),
        ));
    }
}
