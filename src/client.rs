//use std::io::BufReader;
//use std::fs;
use std::sync::Arc;

use capnp::capability::Promise;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use echo_capnp::echo;

use futures::Future;
use tokio_io::AsyncRead;

use rustls::ClientConfig;
use tokio_rustls::TlsConnector;

extern crate webpki;
extern crate webpki_roots;

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

pub fn main() {
    let args: Vec<String> = ::std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} client HOST:PORT", args[0]);
        return;
    }
    try_main(args[2].to_string()).unwrap();
}

pub fn try_main(addr_port: String) -> Result<(), ::capnp::Error> {
    use std::net::ToSocketAddrs;

    let mut core = ::tokio_core::reactor::Core::new()?;
    let handle = core.handle();

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
    let config = TlsConnector::from(Arc::new(config));

    let domain = webpki::DNSNameRef::try_from_ascii_str("localhost").unwrap();

    let socket = ::tokio_core::net::TcpStream::connect(&addr, &handle);
    let tls_handshake = socket.and_then(|socket| {
        socket.set_nodelay(true).unwrap();
        config.connect(domain, socket)
    });

    let stream = core.run(tls_handshake).unwrap();
    let (reader, writer) = stream.split();

    let network = Box::new(twoparty::VatNetwork::new(
        reader,
        writer,
        rpc_twoparty_capnp::Side::Client,
        Default::default(),
    ));
    let mut rpc_system = RpcSystem::new(network, None);
    let echo_client: echo::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
    let rpc_disconnector = rpc_system.get_disconnector();
    handle.spawn(rpc_system.map_err(|e| println!("{}", e)));

    let mut request = echo_client.echo_request();
    request.get().set_input("hello");
    core.run(request.send().promise.and_then(|response| {
        let output = pry!(response.get()).get_output().unwrap();
        println!("{}", output);
        Promise::ok(())
    }))?;

    core.run(rpc_disconnector)?;
    Ok(())
}
