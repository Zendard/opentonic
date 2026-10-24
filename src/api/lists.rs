use crate::ServerState;
use axum::{
    Json,
    extract::{self, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::Json as SqlxJson;
use std::sync::Arc;

#[derive(Serialize, FromRow)]
pub struct List {
    id: i64,
    name: String,
    owner: String,
    users: SqlxJson<Vec<String>>,
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

    let lists: Vec<List> = sqlx::query_as!(
        List,
        r#"
         SELECT id,name,owner,json_group_array(la2.user) as "users!: SqlxJson<Vec<String>>" FROM Lists l 
         LEFT JOIN List_Accesses la ON la.list_id = l.id
         LEFT JOIN List_Accesses la2 ON la.list_id = l.id
         WHERE la.user = ?
         GROUP BY l.id;
     "#,
        user
    )
    .fetch_all(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(lists))
}

#[derive(Deserialize)]
pub struct CreateListReq {
    name: String,
}

pub async fn create_list(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<CreateListReq>,
) -> Result<Json<List>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let list=sqlx::query_as!(
        List,
        r#" 
        INSERT INTO Lists(name,owner) VALUES (?,?);

        INSERT INTO Categories(id,name) VALUES (1,"Uncategorized");

        INSERT INTO List_Accesses(list_id,user) VALUES (last_insert_rowid(),?);

        SELECT id,name,owner,json_group_array(la.user) as "users!: SqlxJson<Vec<String>>" FROM Lists l 
        LEFT JOIN List_Accesses la ON la.list_id = l.id
        WHERE l.id = (SELECT list_id FROM List_Accesses WHERE rowid=last_insert_rowid())
        GROUP BY l.id;
        "#,
        body.name,user,user
    )
    .fetch_one(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(list))
}

pub async fn get_list(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
    extract::Path(list_id): extract::Path<String>,
) -> Result<Json<List>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let list = sqlx::query_as!(
        List,
        r#"
         SELECT id,name,owner,json_group_array(la.user) as "users!: SqlxJson<Vec<String>>" FROM Lists l 
         LEFT JOIN List_Accesses la ON la.list_id = l.id
         WHERE l.id=?
         GROUP BY l.id;
        "#,
        list_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;

    if !list.users.contains(&user.to_string()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(Json(list))
}

#[derive(Deserialize)]
pub struct UpdateListReq {
    name: Option<String>,
    users: Option<Vec<String>>,
}

pub async fn update_list(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<UpdateListReq>,
) -> Result<Json<List>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let list_id: i64 = list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    let owner = sqlx::query!(
        "
        SELECT owner FROM Lists WHERE id = ?
        ",
        list_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?
    .owner;

    if owner != user {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Some(name) = body.name {
        sqlx::query!(
            "
            UPDATE Lists SET name=? WHERE id=?;
            ",
            name,
            list_id
        )
        .execute(&state.db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    if let Some(mut users) = body.users {
        // Make sure the owner always has access
        if !users.contains(&owner) {
            users.push(owner);
        }

        sqlx::query!(
            "
                DELETE FROM List_Accesses WHERE list_id=?
            ",
            list_id
        )
        .execute(&state.db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for user in users {
            sqlx::query!(
                "
                INSERT INTO List_Accesses(list_id,user) VALUES (?,?) ON CONFLICT DO NOTHING;
                ",
                list_id,
                user
            )
            .execute(&state.db_conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    let list = sqlx::query_as!(
        List,
        r#"
         SELECT id,name,owner,json_group_array(la.user) as "users!: SqlxJson<Vec<String>>" FROM Lists l 
         LEFT JOIN List_Accesses la ON la.list_id = l.id
         WHERE l.id=?
         GROUP BY l.id;
        "#,
        list_id
    )
    .fetch_one(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(list))
}

pub async fn delete_list(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<i64>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let list_id: i64 = list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    sqlx::query!(
        "SELECT owner FROM Lists WHERE owner=? AND id=?",
        user,
        list_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let deleted_id = sqlx::query!(
        "
        DELETE FROM List_Accesses WHERE list_id=?;
        DELETE FROM Lists WHERE id=? RETURNING id;
        ",
        list_id,
        list_id
    )
    .fetch_one(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .id
    .ok_or(StatusCode::IM_A_TEAPOT)?;

    Ok(Json(deleted_id))
}
