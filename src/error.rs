#[derive(Debug, thiserror::Error)]
pub(crate) enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse YAML: {0}")]
    Yaml(#[from] serde_saphyr::Error),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum HookError {
    #[error("{0}")]
    Config(#[from] ConfigError),

    #[error("{0}")]
    Input(#[from] crate::input::InputError),

    #[error("failed to parse input for templates: {0}")]
    TemplateParse(String),

    #[error("invalid matcher regex: {0}")]
    Matcher(#[from] regex::Error),

    #[error("command failed with exit code {code}: {command}")]
    CommandFailed { code: i32, command: String },

    #[error("failed to execute command: {command}: {source}")]
    CommandExecution {
        command: String,
        source: std::io::Error,
    },
}

impl HookError {
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            HookError::CommandFailed { .. } | HookError::CommandExecution { .. } => 2,
            _ => 1,
        }
    }
}
