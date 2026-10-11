use an_net::identity::Identity;
use an_net::tls;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, Error, SignatureScheme};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};

fn identity(name: &str) -> Identity {
    let dir = std::env::temp_dir().join(format!("an-net-tls-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    Identity::load_or_create(&dir.join("identity.key")).unwrap()
}

/// 연결 1개를 받아 핸드셰이크 결과(성공 시 상대 SPKI)를 돌려주는 서버.
async fn serve_once(
    server: &Identity,
) -> (
    u16,
    tokio::task::JoinHandle<Result<Vec<u8>, std::io::Error>>,
) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let acceptor = TlsAcceptor::from(tls::server_config(server));
    let task = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await?;
        let mut s = acceptor.accept(tcp).await?;
        let spki = s.get_ref().1.peer_certificates().unwrap()[0].to_vec();
        let mut b = [0u8; 1];
        s.read_exact(&mut b).await?;
        s.write_all(&b).await?;
        Ok(spki)
    });
    (port, task)
}

async fn connect(cfg: Arc<ClientConfig>, port: u16) -> std::io::Result<Vec<u8>> {
    let tcp = TcpStream::connect(("127.0.0.1", port)).await?;
    let name = ServerName::IpAddress(std::net::Ipv4Addr::LOCALHOST.into());
    let mut s = TlsConnector::from(cfg).connect(name, tcp).await?;
    s.write_all(b"x").await?;
    let mut b = [0u8; 1];
    s.read_exact(&mut b).await?;
    Ok(s.get_ref().1.peer_certificates().unwrap()[0].to_vec())
}

#[tokio::test]
async fn pinned_key_connects_and_sees_client_key() {
    let (a, b) = (identity("pin-a"), identity("pin-b"));
    let (port, server) = serve_once(&b).await;
    let seen_server = connect(tls::client_config(&a, Some(b.spki().to_vec())), port)
        .await
        .unwrap();
    assert_eq!(seen_server, b.spki());
    assert_eq!(
        server.await.unwrap().unwrap(),
        a.spki(),
        "서버가 본 클라이언트 키"
    );

    // 첫 연결(고정 없음)도 서버 키를 그대로 보여 준다.
    let (port, server) = serve_once(&b).await;
    assert_eq!(
        connect(tls::client_config(&a, None), port).await.unwrap(),
        b.spki()
    );
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn pinned_mismatch_fails() {
    let (a, b, c) = (identity("mis-a"), identity("mis-b"), identity("mis-c"));
    let (port, server) = serve_once(&b).await;
    assert!(
        connect(tls::client_config(&a, Some(c.spki().to_vec())), port)
            .await
            .is_err()
    );
    assert!(server.await.unwrap().is_err());
}

/// 서버 키는 아무거나 받고 클라이언트 키는 내지 않는 대역 클라이언트.
#[derive(Debug)]
struct NoCheck;

impl ServerCertVerifier for NoCheck {
    fn verify_server_cert(
        &self,
        _: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }
    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

#[tokio::test]
async fn client_without_key_rejected() {
    let b = identity("nokey-b");
    let (port, server) = serve_once(&b).await;
    let cfg =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_protocol_versions(&[&rustls::version::TLS13])
            .unwrap()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCheck))
            .with_no_client_auth();
    assert!(connect(Arc::new(cfg), port).await.is_err());
    assert!(
        server.await.unwrap().is_err(),
        "키 없는 클라이언트를 받았다"
    );
}
