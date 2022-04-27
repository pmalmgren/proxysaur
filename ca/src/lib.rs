/// Contains code related to generating certificates from a certificate authority.
use anyhow::Result;
use directories::ProjectDirs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{io::BufReader, os::unix::prelude::PermissionsExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use tokio::try_join;
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

pub(crate) async fn project_dirs() -> Result<ProjectDirs> {
    let project_dirs = ProjectDirs::from("com", "proxysaur", "proxysaur")
        .ok_or_else(|| anyhow::Error::msg("Could not build project dirs"))?;

    for dir in [
        project_dirs.cache_dir(),
        project_dirs.config_dir(),
        project_dirs.data_dir(),
    ] {
        match tokio::fs::metadata(dir).await {
            Ok(path) => {
                if !path.is_dir() {
                    let error = format!("{:?} exists and is not a directory.", dir);
                    return Err(anyhow::Error::msg(error));
                }
            }
            Err(_) => {
                tokio::fs::create_dir(dir).await?;
            }
        }
    }

    Ok(project_dirs)
}

pub fn default_ca_dir() -> Result<PathBuf> {
    ProjectDirs::from("com", "proxysaur", "proxysaur")
        .ok_or_else(|| anyhow::Error::msg("Could not get project dirs"))
        .map(|project_dir| project_dir.data_dir().to_path_buf())
}

/// Holds a path to the script used to generate and sign certificates.
#[derive(Clone)]
pub struct CertificateAuthority {
    #[allow(unused)]
    project_dirs: ProjectDirs,
    ca_path: PathBuf,
    config_cache: Arc<RwLock<HashMap<String, ServerConfig>>>,
}

pub async fn valid_ca_directory(ca_path: &Path) -> bool {
    let ca_key_path = ca_path.join("myca.key");
    let ca_cert_path = ca_path.join("myca.pem");
    match try_join!(
        tokio::fs::metadata(ca_cert_path),
        tokio::fs::metadata(ca_key_path),
    ) {
        Ok((key_metadata, cert_metadata)) => {
            if !key_metadata.is_file() {
                return false;
            }
            if !cert_metadata.is_file() {
                return false;
            }

            true
        }
        Err(_err) => false,
    }
}

impl CertificateAuthority {
    /// Loads the `generatecert.sh` script which dynamically generates certificate requests.
    pub async fn load(ca_path: PathBuf) -> Result<Self> {
        let script = include_str!("scripts/generatecert.sh");
        let project_dirs = project_dirs().await?;

        if !valid_ca_directory(&ca_path).await {
            let msg = format!("{:?} is not a valid CA directory", ca_path);
            return Err(anyhow::Error::msg(msg));
        }

        {
            let script_dir_str = project_dirs
                .data_dir()
                .to_str()
                .ok_or_else(|| CaError::CustomError("Error parsing script directory.".into()))?
                .to_string();
            let script_path = project_dirs.data_dir().join("generatecert.sh");
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

        let config_cache = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            project_dirs,
            ca_path,
            config_cache,
        })
    }
}

impl CertificateAuthority {
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

    async fn check_for_certs(
        &mut self,
        host: &str,
        cert_path: &Path,
        key_path: &Path,
    ) -> Result<Option<ServerConfig>> {
        if let Some(existing_config) = {
            let cache = self.config_cache.read().await;
            cache.get(host).cloned()
        } {
            return Ok(Some(existing_config));
        }

        let certs = Self::get_certs(cert_path).await;
        let key = Self::get_private_key(key_path).await;

        match (certs, key) {
            (Ok(certs), Ok(key)) => {
                let bundle = CertAndKey { certs, key };
                let config: ServerConfig = bundle.try_into()?;
                Ok(Some(config))
            }
            _ => Ok(None),
        }
    }

    pub async fn build_certs(&mut self, host: &str, port: u16) -> Result<ServerConfig, CaError> {
        let tls_ca_path = self.ca_path.clone();
        let tls_ca_path_str = tls_ca_path
            .to_str()
            .ok_or_else(|| CaError::CustomError("Invalid CA path".into()))?;
        let cert_path = tls_ca_path.join(format!("{}.crt", host));
        let key_path = tls_ca_path.join(format!("{}.key", host));

        if let Some(bundle) = self
            .check_for_certs(host, &cert_path, &key_path)
            .await
            .map_err(|_err| CaError::CustomError("Error checking for certs".into()))?
        {
            return Ok(bundle);
        }

        let port = port.to_string();
        let output = tokio::process::Command::new("generatecert.sh")
            .args([host, port.as_str(), tls_ca_path_str])
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

        let certs = Self::get_certs(cert_path).await?;
        let key = Self::get_private_key(key_path).await?;

        let bundle = CertAndKey { certs, key };

        let config: ServerConfig = bundle.try_into()?;
        Ok(config)
    }
}
