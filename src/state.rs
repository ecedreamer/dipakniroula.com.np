use axum_csrf::CsrfConfig;
use crate::db::{DbPool, PooledConn};
use crate::utils::error::AppError;

use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub csrf_config: CsrfConfig,
}

impl FromRef<AppState> for CsrfConfig {
    fn from_ref(state: &AppState) -> Self {
        state.csrf_config.clone()
    }
}

impl AppState {
    pub async fn get_conn(&self) -> Result<PooledConn, AppError> {
        self.db_pool.get().await.map_err(|e| AppError::Internal(e.to_string()))
    }
}

