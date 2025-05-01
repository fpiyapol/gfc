use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitSource {
    pub url: String,
    pub branch: String,
    pub path: String,
}
