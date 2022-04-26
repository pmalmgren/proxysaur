/// Contains code related to generating certificates from a certificate authority.
use std::error::Error as StdError;
use std::path::Path;
use std::{io::BufReader, os::unix::prelude::PermissionsExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::rustls::{
    Certificate, PrivateKey, ServerConfig, ALL_CIPHER_SUITES, ALL_KX_GROUPS, ALL_VERSIONS,
};

pub mod cli;
pub use cli::*;

#[derive(thiserror::Error, Debug)]
pub enum CaError {
    #[error("Error fetching private key")]
    KeyFetchError,
    #[error("Error generating certificates")]
    GenerateCertificate,
    #[error(transparent)]
    RustlsError(#[from] tokio_rustls::rustls::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Error: {0}")]
    CustomError(String),
}

pub struct CertAndKey {
    pub certs: Vec<Certificate>,
    pub key: PrivateKey,
}

impl TryInto<ServerConfig> for CertAndKey {
    type Error = CaError;

    fn try_into(self) -> Result<ServerConfig, Self::Error> {
        let cfg = ServerConfig::builder()
            .with_cipher_suites(ALL_CIPHER_SUITES)
            .with_kx_groups(&ALL_KX_GROUPS)
            .with_protocol_versions(ALL_VERSIONS)
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(self.certs, self.key)
            .map_err(CaError::from)?;
        Ok(cfg)
    }
}

/// Holds a path to the script used to generate and sign certificates.
pub struct CertificateAuthority {
    // TODO: Use directories = "4.0.1"
    #[allow(unused)]
    script_dir: tempdir::TempDir,
}

impl CertificateAuthority {
    /// Loads the `generatecert.sh` script which dynamically generates certificate requests.
    pub async fn load() -> Result<Self, Box<dyn StdError>> {
        let script = include_str!("scripts/generatecert.sh");
        let script_dir = tempdir::TempDir::new("script").map_err(|error| {
            tracing::error!(%error, "Error creating temporary directory.");
            error
        })?;

        {
            let script_dir_str = script_dir
                .as_ref()
                .to_str()
                .ok_or_else(|| CaError::CustomError("Error parsing script directory.".into()))?
                .to_string();
            let script_path = script_dir.path().join("generatecert.sh");
            let mut file = tokio::fs::File::create(script_path).await?;
            file.write(script.as_bytes()).await.map_err(|error| {
                tracing::error!(%error, "Error writing script file.");
                error
            })?;
            file.set_permissions(std::fs::Permissions::from_mode(0o777))
                .await?;

            let path = std::env::var("PATH").unwrap_or_else(|_| "".to_string());
            std::env::set_var("PATH", format!("{}:{}", path, script_dir_str));
        }

        Ok(Self { script_dir })
    }
}

async fn get_certs<P: AsRef<Path>>(cert_path: P) -> Result<Vec<Certificate>, CaError> {
    let mut f = tokio::fs::File::open(cert_path).await?;
    let fsize: usize = f
        .metadata()
        .await?
        .len()
        .try_into()
        .map_err(|_e| CaError::KeyFetchError)?;
    let mut buf = vec![0u8; fsize];
    let _n_read = f.read(&mut buf).await?;
    let mut bufreader: BufReader<&[u8]> = BufReader::new(&buf);
    let raw_certs = rustls_pemfile::certs(&mut bufreader)?;
    let certs: Vec<Certificate> = raw_certs.into_iter().map(Certificate).collect();
    tracing::debug!(certs_len = %certs.len(), "Parsed certificates.");
    Ok(certs)
}

async fn get_private_key<P: AsRef<Path>>(key_path: P) -> Result<PrivateKey, CaError> {
    let mut f = tokio::fs::File::open(key_path).await?;
    let fsize: usize = f
        .metadata()
        .await?
        .len()
        .try_into()
        .map_err(|_e| CaError::KeyFetchError)?;
    let mut buf = vec![0u8; fsize];
    let _n_read = f.read(&mut buf).await?;
    let mut bufreader: BufReader<&[u8]> = BufReader::new(&buf);
    let raw_keys = rustls_pemfile::rsa_private_keys(&mut bufreader)?;
    tracing::debug!(key_len = %raw_keys.len(), "Parsed private keys.");
    let keys: Vec<PrivateKey> = raw_keys.into_iter().map(PrivateKey).collect();
    Ok(keys[0].clone())
}

async fn check_for_certs(cert_path: &str, key_path: &str) -> Option<CertAndKey> {
    let certs = get_certs(cert_path).await;
    let key = get_private_key(key_path).await;

    match (certs, key) {
        (Ok(certs), Ok(key)) => Some(CertAndKey { certs, key }),
        _ => None,
    }
}

pub async fn build_certs(host: &str, port: u16, tls_ca_path: &str) -> Result<CertAndKey, CaError> {
    let cert_path = format!("{}/{}.crt", tls_ca_path, host);
    let key_path = format!("{}/{}.key", tls_ca_path, host);

    if let Some(bundle) = check_for_certs(&cert_path, &key_path).await {
        tracing::debug!(%cert_path, %key_path, "Found existing certificates.");
        return Ok(bundle);
    }

    let port = port.to_string();
    let output = tokio::process::Command::new("generatecert.sh")
        .args([host, port.as_str(), tls_ca_path])
        .output()
        .await
        .map_err(|error| {
            tracing::error!(%error, "Error running generate certificate script.");
            error
        })?;

    if !output.status.success() {
        return Err(CaError::GenerateCertificate);
    }

    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|_e| CaError::CustomError("Error serializing generatecert.sh output".into()))?
        .trim()
        .trim_matches('\n');

    let parts: Vec<&str> = stdout.split(' ').collect();

    if parts.len() != 2 {
        return Err(CaError::CustomError(
            "Error serializing generatecert.sh output".into(),
        ));
    }
    let (cert_path, key_path): (&str, &str) = (parts[0], parts[1]);

    tracing::info!(%cert_path, %key_path, "Built certificate and key.");

    let certs = get_certs(cert_path).await?;
    let key = get_private_key(key_path).await?;

    Ok(CertAndKey { certs, key })
}
