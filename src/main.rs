#[macro_use]
extern crate capnp_rpc;

mod client;
pub mod echo_capnp;
mod server;

//use std::io::BufReader;
//use std::fs;

/*
fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls::internal::pemfile::certs(&mut reader).unwrap()
}
*/

/*
fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let rsa_keys = {
        let keyfile = fs::File::open(filename).expect("cannot open private key file");
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::rsa_private_keys(&mut reader)
            .expect("file contains invalid rsa private key")
    };

    let pkcs8_keys = {
        let keyfile = fs::File::open(filename).expect("cannot open private key file");
        let mut reader = BufReader::new(keyfile);
        rustls::internal::pemfile::pkcs8_private_keys(&mut reader)
            .expect("file contains invalid pkcs8 private key (encrypted keys not supported)")
    };

    // prefer to load pkcs8 keys
    if !pkcs8_keys.is_empty() {
        pkcs8_keys[0].clone()
    } else {
        assert!(!rsa_keys.is_empty());
        rsa_keys[0].clone()
    }
}
*/

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

#[cfg(test)]
mod tests {
    use super::*;
    use async_std;
    use std::time::Duration;

    #[tokio::test]
    async fn test_rust_client_rust_server() {
        let local = tokio::task::LocalSet::new();
        let server = server::try_main("localhost:31111".to_string());
        local.spawn_local(server);

        let client = client::try_main("localhost:31111".to_string());
        local
            .run_until(async {
                async_std::task::sleep(Duration::from_millis(100)).await;
                client.await.unwrap();
            })
            .await;
    }
}
