use serde::{Serialize, Serializer, ser::SerializeStruct};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid settings: {0}")]
    InvalidSettings(String),
    #[error("not available on this system")]
    Unsupported,
    #[error("{0}")]
    Failed(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl AppError {
    fn kind(&self) -> &'static str {
        match self {
            Self::InvalidSettings(_) => "invalidSettings",
            Self::Unsupported => "unsupported",
            Self::Failed(_) => "failed",
            Self::Io(_) => "io",
            Self::Json(_) => "json",
            Self::Tauri(_) => "tauri",
        }
    }
}

/// Sent to the interface as `{ kind, message }`.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut error = serializer.serialize_struct("AppError", 2)?;
        error.serialize_field("kind", self.kind())?;
        error.serialize_field("message", &self.to_string())?;
        error.end()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
