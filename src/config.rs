use config::Config;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub from_name: String,
    #[serde(default = "default_delay")]
    pub send_delay_ms: u64,
}

fn default_delay() -> u64 {
    2000
}

impl SmtpConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let settings = Config::builder()
            .add_source(config::File::with_name("src/Settings"))
            .build()?;
        let cfg: SmtpConfig = settings.try_deserialize()?;
        Ok(cfg)
    }
}

