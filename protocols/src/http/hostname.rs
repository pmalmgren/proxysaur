use http::Request;
use hyper::Body;

#[derive(Debug, Clone)]
pub struct Hostname {
    pub authority: String,
    pub host: String,
    pub scheme: String,
    pub port: u16,
}

impl TryFrom<&Request<Body>> for Hostname {
    type Error = anyhow::Error;

    fn try_from(req: &Request<Body>) -> Result<Self, Self::Error> {
        let host = req
            .uri()
            .authority()
            .ok_or_else(|| anyhow::Error::msg("Missing authority"))?;
        let authority = host.as_str().to_string();
        let parts: Vec<&str> = host.as_str().split(':').collect();
        if parts.len() != 2 {
            tracing::error!(%authority, "Hostname missing port.");
            return Err(anyhow::Error::msg("Missing port"));
        }
        let port: u16 = match parts[1].parse() {
            Ok(v) => v,
            Err(_error) => {
                tracing::error!(%authority, "Hostname missing port.");
                return Err(anyhow::Error::msg("Missing port"));
            }
        };
        let host = parts[0].to_string();
        Ok(Hostname {
            authority,
            host,
            port,
            scheme: "https".to_string(),
        })
    }
}
