use config::{Config, File, FileFormat};
use once_cell::sync::Lazy;

static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::builder()
        .add_source(File::new("config.toml", FileFormat::Toml))
        .build()
        .expect("Failed to build config")
});

pub fn get_config() -> &'static Config {
    &CONFIG
}
