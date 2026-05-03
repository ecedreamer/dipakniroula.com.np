use axum::Router;
use utoipa::OpenApi;
use utoipa::openapi::security::{SecurityScheme, ApiKey, ApiKeyValue};
use utoipa_swagger_ui::SwaggerUi;
use crate::state::AppState;

pub mod auth;
pub mod v1;

#[derive(OpenApi)]
#[openapi(
    paths(
        v1::blogs::get_blogs,
        v1::blogs::get_blog_by_id,
        v1::experiences::get_experiences,
        v1::messages::get_messages,
    ),
    components(
        schemas(
            v1::blogs::BlogDto,
            v1::experiences::ExperienceDto,
            v1::messages::MessageDto,
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "ApiKeyAuth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
            )
        }
    }
}

pub fn configure_api() -> Router<AppState> {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/v1", v1::v1_routes())
}
