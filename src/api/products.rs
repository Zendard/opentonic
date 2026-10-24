use crate::ServerState;
use axum::{
    Json,
    extract::{self, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct Product {
    id: i64,
    name: String,
    category_id: i64,
}

#[derive(Deserialize)]
pub struct QueryProducts {
    q: Option<String>,
}

#[axum::debug_handler]
pub async fn query_products(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    query: extract::Query<QueryProducts>,
    State(state): State<Arc<ServerState>>,
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

    let sql_query = if let Some(q) = &query.q {
        sqlx::query_as!(
            Product,
            "
        SELECT id, name, category_id FROM Products
        WHERE name LIKE concat('%',?,'%') AND list_id=?;
        ",
            q,
            list_id
        )
        .fetch_all(&state.db_conn)
        .await
    } else {
        sqlx::query_as!(
            Product,
            "
        SELECT id, name,category_id FROM Products
        WHERE list_id=?;
        ",
            list_id
        )
        .fetch_all(&state.db_conn)
        .await
    };

    let products = sql_query.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(products))
}

#[axum::debug_handler]
pub async fn get_product(
    headers: HeaderMap,
    extract::Path((list_id, product_id)): extract::Path<(String, String)>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Product>, StatusCode> {
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
        SELECT id, name,category_id FROM Products
        WHERE id=? AND list_id=?;
        ",
        product_id,
        list_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(products))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductReq {
    name: Option<String>,
    category_id: Option<i64>,
}

#[axum::debug_handler]
pub async fn update_product(
    headers: HeaderMap,
    extract::Path((list_id, product_id)): extract::Path<(String, String)>,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<UpdateProductReq>,
) -> Result<Json<Product>, StatusCode> {
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

    if let Some(name) = body.name {
        sqlx::query!(
            "
        UPDATE Products SET name=? WHERE id=?
        ",
            name,
            product_id
        )
        .execute(&state.db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    if let Some(category_id) = body.category_id {
        sqlx::query!(
            "
        UPDATE Products SET category_id=? WHERE id=?
        ",
            category_id,
            product_id
        )
        .execute(&state.db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let products = sqlx::query_as!(
        Product,
        "
        SELECT id,name,category_id FROM Products
        WHERE id=? AND list_id=?;
        ",
        product_id,
        list_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(products))
}

#[axum::debug_handler]
pub async fn delete_product(
    headers: HeaderMap,
    extract::Path((list_id, product_id)): extract::Path<(String, String)>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<i64>, StatusCode> {
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

    let products = sqlx::query!(
        "
        DELETE FROM Products WHERE id=? RETURNING id;
        ",
        product_id,
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?
    .id;

    Ok(Json(products))
}
