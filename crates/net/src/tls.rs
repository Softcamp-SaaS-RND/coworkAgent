//! TLS 1.3 + raw public key(RFC 7250) 상호 인증. 검증기는 **키 보유 증명만** 한다 — 누구인지(인가)는 연결 뒤
//! `peer_certificates()[0]`(상대 SPKI)로 피어 표에서 판단한다.
use crate::identity::{Identity, fingerprint};
use crate::invite::normalize_fingerprint;
use rustls::client::AlwaysResolvesClientRawPublicKeys;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{
    CryptoProvider, WebPkiSupportedAlgorithms, verify_tls13_signature_with_raw_key,
};
use rustls::pki_types::{CertificateDer, ServerName, SubjectPublicKeyInfoDer, UnixTime};
use rustls::server::AlwaysResolvesServerRawPublicKeys;
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::{
    CertificateError, ClientConfig, DigitallySignedStruct, DistinguishedName, Error, ServerConfig,
    SignatureScheme,
};
use std::sync::Arc;

fn provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// 수신 쪽 — 클라이언트 키 필수, 키는 무엇이든 받는다.
pub fn server_config(id: &Identity) -> Arc<ServerConfig> {
    let p = provider();
    let algs = p.signature_verification_algorithms;
    let cfg = ServerConfig::builder_with_provider(p)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("ring 은 TLS 1.3 을 지원한다")
        .with_client_cert_verifier(Arc::new(AnyClientKey(algs)))
        .with_cert_resolver(Arc::new(AlwaysResolvesServerRawPublicKeys::new(
            id.certified_key(),
        )));
    Arc::new(cfg)
}

/// 서버 키를 무엇에 고정하나.
#[derive(Debug, Clone)]
pub enum Pin {
    /// 고정 없음 — 초대 없이 주소만으로 추가할 때(사람이 지문을 비교한다).
    Any,
    /// 피어 표의 SPKI 바이트.
    Key(Vec<u8>),
    /// 초대에 실린 지문(hex 32자) — 다르면 핸드셰이크에서 끊어 hello 를 보내지 않는다.
    Fingerprint(String),
}

/// 발신 쪽 — `pinned` 가 있으면 서버 키가 그 SPKI 와 같아야 한다. `None` 은 주소 추가(첫 연결)뿐.
pub fn client_config(id: &Identity, pinned: Option<Vec<u8>>) -> Arc<ClientConfig> {
    client_config_pin(id, pinned.map_or(Pin::Any, Pin::Key))
}

pub fn client_config_pin(id: &Identity, pin: Pin) -> Arc<ClientConfig> {
    let p = provider();
    let algs = p.signature_verification_algorithms;
    let cfg = ClientConfig::builder_with_provider(p)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("ring 은 TLS 1.3 을 지원한다")
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(ServerKey { pin, algs }))
        .with_client_cert_resolver(Arc::new(AlwaysResolvesClientRawPublicKeys::new(
            id.certified_key(),
        )));
    Arc::new(cfg)
}

fn verify13(
    algs: &WebPkiSupportedAlgorithms,
    message: &[u8],
    cert: &CertificateDer<'_>,
    dss: &DigitallySignedStruct,
) -> Result<HandshakeSignatureValid, Error> {
    verify_tls13_signature_with_raw_key(
        message,
        &SubjectPublicKeyInfoDer::from(cert.as_ref()),
        dss,
        algs,
    )
}

fn tls12_refused() -> Result<HandshakeSignatureValid, Error> {
    Err(Error::General("TLS 1.2 는 쓰지 않는다".into()))
}

#[derive(Debug)]
struct AnyClientKey(WebPkiSupportedAlgorithms);

impl ClientCertVerifier for AnyClientKey {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }
    fn verify_client_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> Result<ClientCertVerified, Error> {
        if !intermediates.is_empty() {
            return Err(Error::InvalidCertificate(CertificateError::BadEncoding));
        }
        Ok(ClientCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        tls12_refused()
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify13(&self.0, message, cert, dss)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }
    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

#[derive(Debug)]
struct ServerKey {
    pin: Pin,
    algs: WebPkiSupportedAlgorithms,
}

impl ServerCertVerifier for ServerKey {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        if !intermediates.is_empty() {
            return Err(Error::InvalidCertificate(CertificateError::BadEncoding));
        }
        let ok = match &self.pin {
            Pin::Any => true,
            Pin::Key(want) => want.as_slice() == end_entity.as_ref(),
            Pin::Fingerprint(want) => {
                normalize_fingerprint(&fingerprint(end_entity.as_ref())).as_deref()
                    == Some(want.as_str())
            }
        };
        if ok {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(Error::InvalidCertificate(
                CertificateError::ApplicationVerificationFailure,
            ))
        }
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        tls12_refused()
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify13(&self.algs, message, cert, dss)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }
    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}
