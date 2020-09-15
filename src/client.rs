//use std::io::BufReader;
//use std::fs;
use std::sync::Arc;

use crate::echo_capnp::echo;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};

use futures::{AsyncReadExt, FutureExt};

use rustls::ClientConfig;
use tokio_rustls::TlsConnector;

mod danger {
    pub struct NoCertificateVerification {}

    impl rustls::ServerCertVerifier for NoCertificateVerification {
        fn verify_server_cert(
            &self,
            _roots: &rustls::RootCertStore,
            _certs: &[rustls::Certificate],
            _hostname: webpki::DNSNameRef<'_>,
            _ocsp: &[u8],
        ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
            Ok(rustls::ServerCertVerified::assertion())
        }
    }
}

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} client HOST:PORT", args[0]);
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

    //let mut pem = BufReader::new(fs::File::open("test-ca/rsa/ca.cert").unwrap());
    let mut config = ClientConfig::new();
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(danger::NoCertificateVerification {}));
    //config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    /*
    config.root_store.add_pem_file(&mut pem).unwrap();
    config.set_single_client_cert(
        ::load_certs("test-ca/rsa/client.cert"),
        ::load_private_key("test-ca/rsa/client.key"),
    );
    */
    let connector = TlsConnector::from(Arc::new(config));

    let domain = webpki::DNSNameRef::try_from_ascii_str("localhost").unwrap();

    let stream = tokio::net::TcpStream::connect(&addr).await?;
    stream.set_nodelay(true)?;
    let stream = connector.connect(domain, stream).await?;

    let (reader, writer) = tokio_util::compat::Tokio02AsyncReadCompatExt::compat(stream).split();

    let network = Box::new(twoparty::VatNetwork::new(
        reader,
        writer,
        rpc_twoparty_capnp::Side::Client,
        Default::default(),
    ));
    let mut rpc_system = RpcSystem::new(network, None);
    let echo_client: echo::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
    let rpc_disconnector = rpc_system.get_disconnector();
    tokio::task::spawn_local(Box::pin(rpc_system.map(|_| ())));

    let mut request = echo_client.echo_request();
    request.get().set_input("hello");
    let response = request.send().promise.await?;
    let output = response.get()?.get_output().unwrap();
    println!("{}", output);

    rpc_disconnector.await?;
    Ok(())
}
