use crate::ServerState;
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct List {
    id: i64,
    name: String,
    users: Vec<u32>,
    products: Vec<u32>,
}

#[derive(Serialize)]
pub struct GetLists {
    id: i64,
    name: String,
}
pub async fn get_lists(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Vec<GetLists>>, StatusCode> {
    let names = sqlx::query_as!(GetLists, "SELECT id,name FROM Lists")
        .fetch_all(&state.db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(names))
}

#[derive(Deserialize)]
pub struct CreateList {
    name: String,
}
pub async fn create_list(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<CreateList>,
) -> Result<Json<List>, StatusCode> {
    let name = body.name;
    sqlx::query!(
        "INSERT INTO Lists(name) VALUES (?);SELECT * FROM Lists WHERE (id==last_insert_rowid())",
        name
    )
    .fetch_one(&state.db_conn)
    .await
    .map(|rec| {
        Json(List {
            id: rec.id,
            name: rec.name,
            users: Vec::new(),
            products: Vec::new(),
        })
    })
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
