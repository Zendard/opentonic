use crate::{ServerState, api::has_access_to_list};
use axum::{
    Json,
    extract::{self, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct Category {
    id: i64,
    name: String,
    order_index: i64,
    icon_id: Option<String>,
}

#[axum::debug_handler]
pub async fn get_categories(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Vec<Category>>, StatusCode> {
    if !has_access_to_list(
        &state.db_conn,
        list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        &headers,
    )
    .await
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let categories = sqlx::query_as!(
        Category,
        "
        SELECT id,name,order_index,icon_id FROM Categories
        WHERE list_id=?;
        ",
        list_id
    )
    .fetch_all(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(categories))
}

#[axum::debug_handler]
pub async fn get_category(
    headers: HeaderMap,
    extract::Path((list_id, category_id)): extract::Path<(String, String)>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Category>, StatusCode> {
    if !has_access_to_list(
        &state.db_conn,
        list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        &headers,
    )
    .await
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let category = sqlx::query_as!(
        Category,
        "
        SELECT id,name,order_index,icon_id FROM Categories
        WHERE list_id=? AND id=?;
        ",
        list_id,
        category_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(category))
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryReq {
    name: String,
    icon_id: Option<String>,
    order_index: Option<i64>,
}
pub async fn create_category(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<CreateCategoryReq>,
) -> Result<Json<Category>, StatusCode> {
    if !has_access_to_list(
        &state.db_conn,
        list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        &headers,
    )
    .await
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let category = sqlx::query_as!(
        Category,
        "
        INSERT INTO Categories(list_id,name,icon_id,order_index) VALUES (?,?,?,?);

        SELECT id,name,order_index,icon_id FROM Categories
        WHERE list_id=? AND id=last_insert_rowid();
        ",
        list_id,
        body.name,
        body.icon_id,
        body.order_index,
        list_id,
    )
    .fetch_one(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(category))
}
