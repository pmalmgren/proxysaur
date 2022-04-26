use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

use crate::CaError;

pub async fn generate_ca(ca_dir: PathBuf) -> Result<(), CaError> {
    let ca_dir_str = ca_dir
        .to_str()
        .ok_or_else(|| CaError::CustomError("Error building certs".into()))?;
    let config = include_str!("scripts/config");
    let script = include_str!("scripts/generateca.sh");

    {
        let script_path = ca_dir.as_path().join("generateca.sh");
        let config_path = ca_dir.as_path().join("config");
        let mut file = tokio::fs::File::create(script_path.as_path()).await?;
        file.write(script.as_bytes()).await.map_err(|error| {
            tracing::error!(%error, "Error writing script file.");
            error
        })?;
        file.set_permissions(std::fs::Permissions::from_mode(0o777))
            .await?;
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
        return Err(CaError::GenerateCertificate);
    }

    eprintln!("CA initialized in: {}", ca_dir_str);

    Ok(())
}
