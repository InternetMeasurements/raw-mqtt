use std::fmt::Debug;
use std::sync::Arc;
use std::time::SystemTime;
use quinn_rustls::Certificate;
use tokio_rustls::rustls;

use tokio_rustls::rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified};
use tokio_rustls::rustls::{DigitallySignedStruct, Error, SignatureScheme};
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName, UnixTime};

#[derive(Debug)]
pub struct SkipServerVerification;

impl SkipServerVerification {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP256_SHA256
        ]
    }
}

pub struct QuinnSkipServerVerification;

impl QuinnSkipServerVerification {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl quinn_rustls::client::ServerCertVerifier for QuinnSkipServerVerification {
    fn verify_server_cert(
        &self, 
        _end_entity: &Certificate, 
        _intermediates: &[Certificate], 
        _server_name: &quinn_rustls::ServerName, 
        _scts: &mut dyn Iterator<Item=&[u8]>, 
        _ocsp_response: &[u8], 
        _now: SystemTime
    ) -> Result<quinn_rustls::client::ServerCertVerified, quinn_rustls::Error> {
        Ok(quinn_rustls::client::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self, 
        _message: &[u8], 
        _cert: &Certificate, 
        _dss: &quinn_rustls::DigitallySignedStruct
    ) -> Result<quinn_rustls::client::HandshakeSignatureValid, quinn_rustls::Error> {
        Ok(quinn_rustls::client::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &Certificate, 
        _dss: &quinn_rustls::DigitallySignedStruct
    ) -> Result<quinn_rustls::client::HandshakeSignatureValid, quinn_rustls::Error> {
        Ok(quinn_rustls::client::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<quinn_rustls::SignatureScheme> {
        vec![
            quinn_rustls::SignatureScheme::ED25519,
            quinn_rustls::SignatureScheme::ED448,
            quinn_rustls::SignatureScheme::RSA_PSS_SHA512,
            quinn_rustls::SignatureScheme::RSA_PSS_SHA384,
            quinn_rustls::SignatureScheme::RSA_PSS_SHA256,
            quinn_rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            quinn_rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            quinn_rustls::SignatureScheme::ECDSA_NISTP256_SHA256
        ]
    }

    fn request_scts(&self) -> bool {
        false
    }
}