use crate::ServerState;
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct List {
    id: i64,
    name: String,
    owner: String,
}

pub async fn get_lists(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Vec<List>>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let names = sqlx::query_as!(
        List,
        "
         SELECT id,name,owner FROM Lists l 
         JOIN List_Accesses la ON la.list_id = l.id
         WHERE la.user = ?
         GROUP BY l.id
     ",
        user
    )
    .fetch_all(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(names))
}

#[derive(Deserialize)]
pub struct CreateList {
    name: String,
}
#[derive(Serialize)]
pub struct CreateListOutput {
    id: i64,
    name: String,
}
pub async fn create_list(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<CreateList>,
) -> Result<Json<CreateListOutput>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let name = body.name;
    sqlx::query!(
        "
        INSERT INTO Lists(name,owner) VALUES (?,?);
        INSERT INTO List_Accesses(list_id,user) VALUES (last_insert_rowid(),?);
        SELECT * FROM List_Accesses la JOIN Lists l ON la.list_id = l.id WHERE la.rowid==last_insert_rowid();
        ",
        name,user,user
    )
    .fetch_one(&state.db_conn)
    .await
    .map(|rec| {
        Json(CreateListOutput{
            id: rec.id,
            name: rec.name,

        })
    })
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
