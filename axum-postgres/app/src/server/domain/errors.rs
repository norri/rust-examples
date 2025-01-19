use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ErrorResponse {
    #[schema(example = "internal error")]
    pub error: String,
}
