//! Optional TLS termination for the one listener.
//!
//! tonic can do TLS itself, and we deliberately do not use it: its acceptor
//! pushes `h2` and nothing else into the ALPN list, with no way to add
//! `http/1.1`. That was harmless while the port carried only gRPC, but this
//! socket now also serves the web UI and the inbound webhook ingress, and an
//! HTTP/1.1-only client that offers ALPN — `curl --http1.1`, most webhook
//! senders — would get a `no_application_protocol` alert instead of a response.
//!
//! So we build the `rustls::ServerConfig` here, advertise both protocols, and
//! hand tonic a stream of already-decrypted connections. Everything downstream
//! (protocol detection, the router, graceful shutdown) is unchanged: hyper
//! sniffs the HTTP/2 preface on the plaintext side, so a client that negotiated
//! no ALPN at all still lands on the right protocol.

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

/// How many completed handshakes may queue up before the accept loop slows down.
const HANDSHAKE_BACKLOG: usize = 128;

fn tls_error(context: &str, error: impl std::fmt::Display) -> StartupError {
    StartupError::Tls(format!("{context}: {error}"))
}

/// Read the certificate chain and key, and build an acceptor that offers both
/// HTTP/2 and HTTP/1.1.
pub fn acceptor(tls: &TlsConfig) -> Result<TlsAcceptor, StartupError> {
    // The *slice* PEM helpers are unconditional, while `from_pem_file` and
    // `pem_file_iter` sit behind rustls-pki-types' `std` feature — reading the
    // bytes ourselves keeps this from depending on who else enables it.
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

    // Name the provider instead of relying on the process-level default, which
    // rustls 0.23 panics on when it cannot pick one unambiguously.
    let mut config = ServerConfig::builder_with_provider(Arc::new(ring::default_provider()))
        .with_safe_default_protocol_versions()
        .map_err(|e| tls_error("selecting TLS protocol versions", e))?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| tls_error("loading the certificate and key", e))?;

    // h2 first, so browsers and gRPC clients still prefer it.
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// Accept TCP connections and yield the ones that finish a TLS handshake.
///
/// Each handshake runs in its own task on purpose. Doing them inline — the
/// obvious `stream! { acceptor.accept(listener.accept().await?).await }` — lets
/// one slow or hostile peer hold the accept loop and stall every other client.
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
                // Per-connection accept errors (fd limits, a peer that vanished)
                // are not a reason to stop listening.
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
                    // A failed handshake is a client problem. Drop it here rather
                    // than forwarding an Err that the server would only log again.
                    Err(error) => tracing::debug!(%peer, %error, "tls handshake failed"),
                }
            });
        }
    });

    Ok(ReceiverStream::new(rx))
}
