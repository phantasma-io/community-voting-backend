use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErr {
    pub err: String
}

impl From<anyhow::Error> for ApiErr {
    fn from(value: anyhow::Error) -> Self {
        Self {
            err: value.to_string()
        }
    }
}

impl axum::response::IntoResponse for ApiErr {
    fn into_response(self) -> axum::response::Response {
        let status = axum::http::StatusCode::from_u16(500).unwrap();
        let body = serde_json::to_string(&self).unwrap_or_default();
        (status, body).into_response()
    }
}
