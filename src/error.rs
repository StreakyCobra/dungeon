use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Message(String),
    Io(std::io::Error),
    Subprocess(i32, String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Subprocess(code, _) => *code,
            _ => 1,
        }
    }

    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Message(msg) => write!(f, "{}", msg),
            AppError::Io(err) => write!(f, "{}", err),
            AppError::Subprocess(_, msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        AppError::Message(err.to_string())
    }
}
