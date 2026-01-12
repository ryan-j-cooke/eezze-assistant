use axum::Router;

use crate::api::chat::register_chat_routes;
use crate::api::index::register_embedding_routes;
use crate::api::models::register_model_routes;

pub fn register_api_routes(app: Router) -> Router {
    let app = register_model_routes(app);
    let app = register_chat_routes(app);
    let app = register_embedding_routes(app);
    app
}
