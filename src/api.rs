use crate::ServerState;
use axum::{
    Json,
    extract::{self, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

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

#[derive(Serialize)]
pub struct FetchList {
    name: String,
    owner: String,
    users: HashSet<String>,
    list_items: Vec<ListItem>,
}

#[derive(Serialize, Debug)]
pub struct ListItem {
    id: i64,
    name: String,
    checked: bool,
    category: Option<String>,
    added_by: String,
    added_on: Option<String>,
}

pub async fn fetch_list(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
    extract::Path(list_id): extract::Path<String>,
) -> Result<Json<FetchList>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let items = sqlx::query_as!(
        ListItem,
        "
        SELECT li.id,p.name as name,c.name as category,added_by,datetime(added_on) as added_on,checked FROM ListItems li
        JOIN Products p ON p.id = li.product_id
        LEFT JOIN Categories c ON c.id = p.category_id 
        WHERE li.list_id = ?;
        ",list_id
    )
    .fetch_all(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let list = sqlx::query!(
        "
        SELECT l.name,l.owner,GROUP_CONCAT(la.user) AS users FROM Lists l
        JOIN List_Accesses la ON la.list_id == l.id WHERE l.id=?;
        ",
        list_id
    )
    .fetch_one(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users_set: HashSet<String> = list
        .users
        .unwrap()
        .split(",")
        .map(|str| str.to_string())
        .collect();

    if !users_set.contains(user) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(Json(FetchList {
        name: list.name.unwrap(),
        owner: list.owner.unwrap(),
        users: users_set,
        list_items: items,
    }))
}

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
            INSERT INTO Products(name) VALUES (?);
            SELECT id FROM Products WHERE id==last_insert_rowid();
            ",
            body.name
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

#[derive(Deserialize)]
pub struct AddUserToList {
    user: String,
}

pub async fn add_user_to_list(
    headers: HeaderMap,
    extract::Path(list_id): extract::Path<String>,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<AddUserToList>,
) -> Result<Json<String>, StatusCode> {
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

    sqlx::query!(
        "
        INSERT INTO List_Accesses(list_id,user) VALUES (?,?) ON CONFLICT DO NOTHING;
        ",
        list_id,
        body.user
    )
    .execute(&state.db_conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(body.user.to_string()))
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

    // Check if user has access to list

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
