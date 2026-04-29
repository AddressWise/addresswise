use ntex::http::{StatusCode, body::Body, header};
use ntex::web::{DefaultError, HttpRequest, HttpResponse, WebResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("no matching address found")]
    NotFound,
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("migration error")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("index error")]
    Tantivy(#[from] tantivy::TantivyError),
    #[error("index query error")]
    TantivyQuery(#[from] tantivy::query::QueryParserError),
    #[error("server error")]
    Io(#[from] std::io::Error),
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    error: ErrorBody<'a>,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: String,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::NotFound => "not_found",
            Self::Database(_)
            | Self::Migration(_)
            | Self::Io(_)
            | Self::Tantivy(_)
            | Self::TantivyQuery(_)
            | Self::Internal(_) => "internal_error",
        }
    }

    fn public_message(&self) -> String {
        match self {
            Self::BadRequest(message) => message.clone(),
            Self::NotFound => "no matching address found".to_string(),
            Self::Database(_)
            | Self::Migration(_)
            | Self::Io(_)
            | Self::Tantivy(_)
            | Self::TantivyQuery(_)
            | Self::Internal(_) => "internal server error".to_string(),
        }
    }
}

impl WebResponseError<DefaultError> for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Database(_)
            | Self::Migration(_)
            | Self::Io(_)
            | Self::Tantivy(_)
            | Self::TantivyQuery(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        if matches!(self, Self::Database(_) | Self::Migration(_) | Self::Io(_)) {
            tracing::error!(error = ?self, "request failed");
        }

        let body = ErrorResponse {
            error: ErrorBody {
                code: self.code(),
                message: self.public_message(),
            },
        };
        let json = serde_json::to_string(&body).unwrap_or_else(|_| {
            r#"{"error":{"code":"internal_error","message":"internal server error"}}"#.to_string()
        });

        let mut response = HttpResponse::new(self.status_code());
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        response.set_body(Body::from(json))
    }
}
