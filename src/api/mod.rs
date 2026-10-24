use axum::http::HeaderMap;

pub mod categories;
pub mod list_items;
pub mod lists;
pub mod products;

async fn has_access_to_list(db_conn: &sqlx::SqlitePool, list_id: i64, headers: &HeaderMap) -> bool {
    let user = headers.get("X-Forwarded-User");
    if user.is_none() {
        return false;
    }
    let user = user.unwrap().to_str();

    if user.is_err() {
        return false;
    }
    let user = user.unwrap();

    sqlx::query!(
        "
        SELECT rowid FROM List_Accesses WHERE list_id=? AND user=?;
        ",
        list_id,
        user
    )
    .fetch_optional(db_conn)
    .await
    .is_ok_and(|i| i.is_some())
}
