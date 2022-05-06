use anyhow::Result;
use openssl::{asn1::Asn1Time, x509::X509};
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tokio::io::AsyncWriteExt;

use crate::{init_project_dirs, valid_ca_directory, CaError};

const CERT_EXTENSIONS: &[&str] = &["crt", "key", "pem", "csr", "ext", "srl", "sh"];

async fn clear_ca_directory(ca_dir: &Path) -> Result<()> {
    let mut files = tokio::fs::read_dir(&ca_dir).await?;
    while let Ok(Some(file)) = files.next_entry().await {
        let path = file.path();
        let should_delete = if path.ends_with("config") {
            true
        } else if let Some(Some(ext)) = file.path().extension().map(|path| path.to_str()) {
            CERT_EXTENSIONS.contains(&ext)
        } else {
            false
        };

        if should_delete {
            tokio::fs::remove_file(path).await?;
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn print_ca_instructions(path: &Path) {
    let cert_path = path.join("myca.crt");
    eprintln!("Root Certificate: {:#?}", cert_path);
    eprintln!("To trust this certificate, run: ");
    eprintln!(
        "sudo cp {:#?} /usr/local/share/ca-certificates/extra",
        cert_path
    );
    eprintln!("sudo update-ca-certificates");
    eprintln!("To use in a browser, read more here: https://proxysaur.us/ca#trusting-the-root-certificate-in-your-browser");
}

#[cfg(target_os = "macos")]
fn print_ca_instructions(path: &Path) {
    let cert_path = path.join("myca.crt");
    eprintln!("Root Certificate: {:#?}", cert_path);
    eprintln!("To trust this certificate, run: ");
    eprintln!(
        "security add-trusted-cert -d -r trustRoot -k $HOME/Library/Keychains/login.keychain {#:?}",
        cert_path
    );
    eprintln!("To use in a browser, read more here: https://proxysaur.us/ca#trusting-the-root-certificate-in-your-browser");
}

pub async fn generate_ca(path: Option<PathBuf>, force_overwrite: bool) -> Result<PathBuf> {
    let ca_dir = match path {
        Some(ca_dir) => ca_dir,
        None => {
            let project_dirs = init_project_dirs().await?;
            tracing::debug!(?project_dirs, "Using project dirs");
            project_dirs.data_dir().to_path_buf()
        }
    };

    match (valid_ca_directory(&ca_dir).await, force_overwrite) {
        (true, true) => clear_ca_directory(&ca_dir).await?,
        (true, false) => {
            let cert_path = ca_dir.join("myca.crt");
            let cert_bytes = tokio::fs::read(&cert_path).await?;
            let cert = X509::from_pem(&cert_bytes)?;

            let unix_ts = chrono::Local::now().timestamp();
            let now = Asn1Time::from_unix(unix_ts)?;

            let after = cert.not_after();
            let before = cert.not_before();

            if now > after {
                eprintln!("Expired certificate. Rebuilding directory.");
                clear_ca_directory(&ca_dir).await?;
            } else if now < before {
                eprintln!("Certificate isn't active yet.");
            } else {
                eprintln!("Using existing CA dir: {:?}", ca_dir);
                print_ca_instructions(&ca_dir);
                return Ok(ca_dir);
            }
        }
        _ => {}
    };

    let ca_dir_str = ca_dir
        .to_str()
        .ok_or_else(|| CaError::CustomError("Error building certs".into()))?;
    let config = include_str!("scripts/config.conf");
    let script = include_str!("scripts/generateca.sh");

    {
        let script_path = ca_dir.join("generateca.sh");
        let config_path = ca_dir.join("config");
        tracing::debug!(?script_path, "Creating script file.");
        let mut file = tokio::fs::File::create(script_path.as_path()).await?;
        file.write(script.as_bytes()).await.map_err(|error| {
            tracing::error!(%error, "Error writing script file.");
            error
        })?;
        file.set_permissions(std::fs::Permissions::from_mode(0o777))
            .await?;
        tracing::debug!(?config, "Creating config file.");
        let mut file = tokio::fs::File::create(config_path.as_path()).await?;
        file.write(config.as_bytes()).await.map_err(|error| {
            tracing::error!(%error, "Error writing config file.");
            error
        })?;

        let path = std::env::var("PATH").unwrap_or_else(|_| "".to_string());
        std::env::set_var("PATH", format!("{}:{}", path, ca_dir_str));
    }

    let output = tokio::process::Command::new("generateca.sh")
        .args([ca_dir_str])
        .output()
        .await
        .map_err(|error| {
            tracing::error!(%error, "Error running generate certificate script.");
            error
        })?;

    if !output.status.success() {
        return Err(anyhow::Error::from(CaError::GenerateCertificate));
    }

    print_ca_instructions(&ca_dir);

    Ok(ca_dir.to_path_buf())
}
