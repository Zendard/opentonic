use crate::{ServerState, api::has_access_to_list};
use axum::{
    Json,
    extract::{self, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Debug)]
pub struct ListItem {
    id: i64,
    name: String,
    checked: bool,
    product_id: i64,
    added_by: String,
    added_on: String,
}

pub async fn get_list_items(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<Vec<ListItem>>, StatusCode> {
    if !has_access_to_list(
        &state.db_conn,
        list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        &headers,
    )
    .await
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let list_items = sqlx::query_as!(
        ListItem,
        r#"
        SELECT li.id, p.name, added_by, datetime(added_on) as "added_on!", p.id as product_id, checked FROM ListItems li
        JOIN Products p ON p.id=product_id
        WHERE li.list_id=?;
        "#,
        list_id
    )
    .fetch_all(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(list_items))
}

#[derive(Deserialize)]
pub struct AddListItemReq {
    name: String,
}
pub async fn create_list_item(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<AddListItemReq>,
) -> Result<Json<ListItem>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if !has_access_to_list(
        &state.db_conn,
        list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        &headers,
    )
    .await
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

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

    let list_item=sqlx::query_as!(
        ListItem,
        r#"
        INSERT INTO ListItems(product_id,list_id,added_by) VALUES (?,?,?);

        SELECT li.id, p.name, added_by, datetime(added_on) as "added_on!", p.id as product_id, checked FROM ListItems li
        JOIN Products p ON p.id=product_id
        WHERE li.id=last_insert_rowid();

        "#,
        product_id,
        list_id,
        user
    )
    .fetch_one(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(list_item))
}

pub async fn get_list_item(
    headers: HeaderMap,
    extract::Path((list_id, list_item_id)): extract::Path<(String, String)>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<ListItem>, StatusCode> {
    if !has_access_to_list(
        &state.db_conn,
        list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?,
        &headers,
    )
    .await
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let list_item = sqlx::query_as!(
        ListItem,
        r#"
        SELECT li.id, p.name, added_by, datetime(added_on) as "added_on!", p.id as product_id, checked FROM ListItems li
        JOIN Products p ON p.id=product_id
        WHERE li.id=? AND li.list_id=?;
        "#,
        list_item_id,list_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(list_item))
}

#[derive(Deserialize)]
pub struct UpdateListItemReq {
    checked: bool,
}
pub async fn update_list_item(
    headers: HeaderMap,
    extract::Path((list_id, list_item_id)): extract::Path<(String, String)>,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<UpdateListItemReq>,
) -> Result<Json<ListItem>, StatusCode> {
    let list_id: i64 = list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let list_item_id: i64 = list_item_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    if !has_access_to_list(&state.db_conn, list_id, &headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let list_item = sqlx::query_as!(
        ListItem,
        r#"
            UPDATE ListItems SET checked=? WHERE id=?;
        SELECT li.id, p.name, added_by, datetime(added_on) as "added_on!", p.id as product_id, checked FROM ListItems li
        JOIN Products p ON p.id=product_id
        WHERE li.id=? AND li.list_id=?;

            "#,
        body.checked,
        list_item_id,
        list_item_id,
        list_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(list_item))
}

pub async fn delete_list_item(
    headers: HeaderMap,
    extract::Path((list_id, list_item_id)): extract::Path<(String, String)>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<i64>, StatusCode> {
    let list_item_id: i64 = list_item_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let list_id: i64 = list_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    if !has_access_to_list(&state.db_conn, list_id, &headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let deleted_id = sqlx::query!(
        "DELETE FROM ListItems WHERE id=? AND list_id=? RETURNING id",
        list_item_id,
        list_id
    )
    .fetch_optional(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?
    .id;

    Ok(Json(deleted_id))
}
