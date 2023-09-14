use rustls::OwnedTrustAnchor;

pub fn create_tls_config() -> tokio_postgres_rustls::MakeRustlsConnect {
    let mut root_cert_store = rustls::RootCertStore::empty();
    root_cert_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|c| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(c.subject, c.spki, c.name_constraints)
    }));

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    tokio_postgres_rustls::MakeRustlsConnect::new(config)
}
