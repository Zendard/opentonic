use crate::ServerState;
use axum::{
    Json,
    extract::{self, Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

#[derive(Serialize)]
pub struct Product {
    name: String,
    category_name: Option<String>,
}

#[derive(Deserialize)]
pub struct ProductQuery {
    prompt: String,
}

#[axum::debug_handler]
pub async fn query_products(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ProductQuery>,
) -> Result<Json<Vec<Product>>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let list_id: i64 = list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    sqlx::query!(
        "SELECT user FROM List_Accesses WHERE user=? AND list_id=?",
        user,
        list_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let products = sqlx::query_as!(
        Product,
        "
        SELECT p.name as name,c.name as category_name FROM Products p
        JOIN Categories c ON p.category_id=c.id WHERE p.name LIKE concat('%',?,'%') AND p.list_id=?;
        ",
        query.prompt,
        list_id
    )
    .fetch_all(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(products))
}
