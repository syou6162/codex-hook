#[derive(Debug, thiserror::Error)]
pub(crate) enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse YAML: {0}")]
    Yaml(#[from] serde_saphyr::Error),
}
