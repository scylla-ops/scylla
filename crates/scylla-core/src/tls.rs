//! Not tonic's TLS: its acceptor offers only `h2` in ALPN, so an HTTP/1.1 client (webhook senders, `curl --http1.1`) would get `no_application_protocol`.
//! Both protocols are advertised here and tonic receives decrypted streams; hyper sniffs the preface.

use crate::config::TlsConfig;
use crate::error::StartupError;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::rustls::crypto::ring;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::server::TlsStream;
use tokio_stream::wrappers::ReceiverStream;

const HANDSHAKE_BACKLOG: usize = 128;

fn tls_error(context: &str, error: impl std::fmt::Display) -> StartupError {
    StartupError::Tls(format!("{context}: {error}"))
}

pub fn acceptor(tls: &TlsConfig) -> Result<TlsAcceptor, StartupError> {
    // The slice PEM helpers are unconditional; `from_pem_file` sits behind a feature others may not enable.
    let cert_pem = std::fs::read(&tls.cert)
        .map_err(|e| tls_error(&format!("reading {}", tls.cert.display()), e))?;
    let key_pem = std::fs::read(&tls.key)
        .map_err(|e| tls_error(&format!("reading {}", tls.key.display()), e))?;

    let certs = CertificateDer::pem_slice_iter(&cert_pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| tls_error("parsing the certificate chain", e))?;
    if certs.is_empty() {
        return Err(tls_error(
            "parsing the certificate chain",
            format!("{} contains no certificate", tls.cert.display()),
        ));
    }
    let key = PrivateKeyDer::from_pem_slice(&key_pem)
        .map_err(|e| tls_error("parsing the private key", e))?;

    // rustls 0.23 panics when it cannot pick a process-level provider unambiguously.
    let mut config = ServerConfig::builder_with_provider(Arc::new(ring::default_provider()))
        .with_safe_default_protocol_versions()
        .map_err(|e| tls_error("selecting TLS protocol versions", e))?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| tls_error("loading the certificate and key", e))?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// One task per handshake: inline, a slow or hostile peer would stall the accept loop.
pub async fn incoming(
    address: SocketAddr,
    acceptor: TlsAcceptor,
) -> Result<ReceiverStream<Result<TlsStream<TcpStream>, std::io::Error>>, StartupError> {
    let listener = TcpListener::bind(address)
        .await
        .map_err(|e| tls_error(&format!("bind {address}"), e))?;

    let (tx, rx) = tokio::sync::mpsc::channel(HANDSHAKE_BACKLOG);
    tokio::spawn(async move {
        loop {
            let (socket, peer) = match listener.accept().await {
                Ok(accepted) => accepted,
                Err(error) => {
                    tracing::debug!(%error, "tcp accept failed");
                    continue;
                }
            };
            let acceptor = acceptor.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                match acceptor.accept(socket).await {
                    Ok(stream) => {
                        let _ = tx.send(Ok(stream)).await;
                    }
                    Err(error) => tracing::debug!(%peer, %error, "tls handshake failed"),
                }
            });
        }
    });

    Ok(ReceiverStream::new(rx))
}
