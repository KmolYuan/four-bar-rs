use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use rcgen::generate_simple_self_signed;
use std::{env::current_dir, fs::write};

pub(crate) fn ssl() -> SslAcceptorBuilder {
    let current = current_dir().expect("cannot locate executable");
    let key = current.join("key.pem");
    let cert = current.join("cert.pem");
    if !key.is_file() || !cert.is_file() {
        let names = vec!["localhost".to_string()];
        let crt = generate_simple_self_signed(names).unwrap();
        write(&cert, crt.serialize_pem().unwrap()).expect("create cert.pem error");
        write(&key, crt.serialize_private_key_pem()).expect("create key.pem error");
    }
    let mut ssl = SslAcceptor::mozilla_intermediate(SslMethod::tls()).expect("SSL builder failed");
    ssl.set_private_key_file(key, SslFiletype::PEM).unwrap();
    ssl.set_certificate_chain_file(cert).unwrap();
    ssl
}
