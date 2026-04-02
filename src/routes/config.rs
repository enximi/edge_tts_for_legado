use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};

use crate::{
    error::AppError,
    http::auth::verify_query_token,
    legado::{ConfigQuery, LegadoConfig},
    services::legado::build_legado_config,
    state::AppState,
};

pub async fn config_endpoint(
    State(state): State<AppState>,
    Query(query): Query<ConfigQuery>,
    headers: HeaderMap,
) -> Result<Json<LegadoConfig>, AppError> {
    verify_query_token(state.config.as_ref(), query.token.as_deref())?;
    Ok(Json(build_legado_config(state.config.as_ref(), &headers)))
}
