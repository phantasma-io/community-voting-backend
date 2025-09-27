
use utoipa::OpenApi;

use crate::server::routes::vote;

#[derive(OpenApi)]
#[openapi(
    modifiers(),
    nest(
        (path="/vote", api = vote::ApiDoc),
    ),
    tags()
)]
pub struct ApiDoc;

