use std::{path::Path, sync::Arc};

use bitcoinsuite_error::{Result, WrapErr};
use rustls::{
    ClientConfig, DangerousClientConfig, RootCertStore, ServerCertVerified, ServerCertVerifier,
    TLSError,
};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};

use crate::{bchd_grpc::bchrpc_client::BchrpcClient, BchdError};

struct NopCertVerifier;

impl ServerCertVerifier for NopCertVerifier {
    fn verify_server_cert(
        &self,
        _roots: &RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> std::result::Result<ServerCertVerified, TLSError> {
        Ok(ServerCertVerified::assertion())
    }
}

const ALPN_H2: &[u8] = b"h2";
pub async fn connect_bchd(
    url: String,
    cert_path: impl AsRef<Path>,
) -> Result<BchrpcClient<Channel>> {
    use std::fs;
    use std::io::Read;
    let mut cert_file = fs::File::open(cert_path).wrap_err(BchdError::CertFile)?;
    let mut cert = Vec::new();
    cert_file
        .read_to_end(&mut cert)
        .wrap_err(BchdError::CertFile)?;
    let mut config = ClientConfig::new();
    config.set_protocols(&[ALPN_H2.to_vec()]);
    let mut dangerous_config = DangerousClientConfig { cfg: &mut config };
    dangerous_config.set_certificate_verifier(Arc::new(NopCertVerifier));
    let tls_config = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(&cert))
        .rustls_client_config(config);
    let url = Endpoint::from_shared(url).unwrap();
    let endpoint = url.tls_config(tls_config).unwrap();
    let bchd = BchrpcClient::connect(endpoint).await.unwrap();
    Ok(bchd)
}
