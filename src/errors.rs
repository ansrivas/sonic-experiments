use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SonicErrors {
    #[error("File operation error {source}")]
    IoError {
        #[from]
        source: io::Error,
    },

    #[error("Failed to parse the configration from env: `{0}`")]
    InvalidConfError(String),

    #[error("ResponseDeserError : {source}")]
    ResponseDeserError {
        #[from]
        source: serde_json::Error,
    },

    #[error("UnsupportedHTTPMethod: {0}")]
    UnsupportedHTTPMethod(String),

    #[error("Database operation error {source}")]
    SqlX {
        #[from]
        source: sqlx::Error,
    },

    #[error("Database migration error {source}")]
    SqlXMigration {
        #[from]
        source: sqlx::migrate::MigrateError,
    },
    #[error("Custom Error {0}")]
    Custom(String),

    #[error("Sonic channel error {source}")]
    Sonic {
        #[from]
        source: sonic_channel::result::Error,
    },

    #[error("Template Render Error {0}")]
    Render(#[from] askama::Error),
}

/// Optionally convert all these errors above to actix-web errors
/// so that we can return them directly from the corresponding
/// HTTP handlers.
impl From<SonicErrors> for actix_web::error::Error {
    fn from(e: SonicErrors) -> actix_web::error::Error {
        match e {
            SonicErrors::SqlX { source } => {
                actix_web::error::ErrorUnprocessableEntity(format!("DB errors {:?}", source))
            }
            _ => actix_web::error::ErrorInternalServerError(format!(
                "Internal Server Eroor. {:?}",
                e.to_string()
            )),
        }
    }
}
