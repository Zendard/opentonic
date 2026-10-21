use crate::ServerState;
use axum::{
    Json,
    extract::{self, Query, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

#[derive(Deserialize)]
pub struct AddListItem {
    name: String,
}
pub async fn add_list_item(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<AddListItem>,
) -> Result<Json<i64>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let product_id = sqlx::query!(
        "
        SELECT id FROM Products WHERE name=?;
        ",
        body.name
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let product_id = if let Some(product_id) = product_id {
        product_id.id
    } else {
        sqlx::query!(
            "
            INSERT INTO Products(name,list_id) VALUES (?,?);
            SELECT id FROM Products WHERE id==last_insert_rowid();
            ",
            body.name,
            list_id
        )
        .fetch_one(&state.db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .id
    };

    sqlx::query!(
        "
        INSERT INTO ListItems(product_id,list_id,added_by) VALUES (?,?,?);
        SELECT id FROM ListItems WHERE id = last_insert_rowid();
        ",
        product_id,
        list_id,
        user
    )
    .fetch_one(&state.db_conn)
    .await
    .map(|rec| Json(rec.id))
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Deserialize)]
pub struct CheckListItemParam {
    checked: bool,
}
pub async fn check_list_item(
    headers: HeaderMap,
    extract::Path(list_item_id): extract::Path<String>,
    extract::Query(param): extract::Query<CheckListItemParam>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<i64>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let list_item_id: i64 = list_item_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    sqlx::query!(
        "
        SELECT la.user FROM ListItems li
        JOIN Lists l ON li.list_id = l.id
        JOIN List_Accesses la ON l.id = la.list_id
        WHERE li.id=? AND la.user = ?
        ",
        list_item_id,
        user
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    sqlx::query!(
        "
        UPDATE ListItems SET checked=? WHERE id=?;
        ",
        param.checked,
        list_item_id
    )
    .execute(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(list_item_id))
}

pub async fn delete_list_item(
    headers: HeaderMap,
    extract::Path(list_item_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<i64>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let list_item_id: i64 = list_item_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    sqlx::query!(
        "SELECT user FROM List_Accesses WHERE user=? AND list_id=(SELECT list_id FROM ListItems WHERE id=?)",
        user,
        list_item_id
    ).fetch_optional(&state.db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let deleted_id = sqlx::query!(
        "DELETE FROM ListItems WHERE id=? RETURNING id",
        list_item_id
    )
    .fetch_one(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .id;

    Ok(Json(deleted_id))
}
