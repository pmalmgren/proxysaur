use anyhow::Result;
use std::{os::unix::fs::PermissionsExt, path::PathBuf};
use tokio::io::AsyncWriteExt;

use crate::{project_dirs, valid_ca_directory, CaError};

const CERT_EXTENSIONS: &[&str] = &["crt", "key", "pem", "csr", "ext", "srl", "sh"];

pub async fn generate_ca(path: Option<PathBuf>, force_overwrite: bool) -> Result<PathBuf> {
    let ca_dir = match path {
        Some(ca_dir) => ca_dir,
        None => {
            let project_dirs = project_dirs().await?;
            tracing::debug!(?project_dirs, "Using project dirs");
            project_dirs.data_dir().to_path_buf()
        }
    };

    match (valid_ca_directory(&ca_dir).await, force_overwrite) {
        (true, true) => {
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
        }
        (true, false) => {
            eprintln!("Refusing to over write existing CA dir: {:?}", ca_dir);
            return Ok(ca_dir);
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

    Ok(ca_dir.to_path_buf())
}
