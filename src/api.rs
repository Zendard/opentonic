use crate::ServerState;
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

#[derive(Serialize)]
pub struct List {
    id: i64,
    name: String,
    users: String,
}

#[derive(Serialize)]
pub struct GetLists {
    id: i64,
    name: String,
    users: HashSet<String>,
}
pub async fn get_lists(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Vec<GetLists>>, StatusCode> {
    let names = sqlx::query_as!(List, "
        SELECT id,name,GROUP_CONCAT(la.user) AS users FROM Lists l JOIN List_Accesses la ON la.list_id = l.id GROUP BY l.id
    ")
    .fetch_all(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
   .iter().map(|rec| GetLists{id:rec.id, name:rec.name.clone(),users:rec.users.split(",").map(|i| i.to_string()).collect()}).collect();
    Ok(Json(names))
}

#[derive(Deserialize)]
pub struct CreateList {
    name: String,
}
#[derive(Serialize)]
pub struct CreateListOutput{
    id:i64,
    name:String,
}
pub async fn create_list(
    State(state): State<Arc<ServerState>>,
    Json(body): Json<CreateList>,
) -> Result<Json<CreateListOutput>, StatusCode> {
    let name = body.name;
    sqlx::query!(
        "
        INSERT INTO Lists(name) VALUES (?);
        INSERT INTO List_Accesses(list_id,user) VALUES (last_insert_rowid(),'test');
        SELECT * FROM List_Accesses la JOIN Lists l ON la.list_id = l.id WHERE la.rowid==last_insert_rowid() ;
        ",
        name
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
