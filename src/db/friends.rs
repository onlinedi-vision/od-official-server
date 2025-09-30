use crate::db::statics;
use chrono::{DateTime, Utc};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn add_friend(
    session: &scylla::client::session::Session,
    user: String,
    friend: String,
) -> Option<Result<()>> {
    let now: DateTime<Utc> = Utc::now();

    let res = session
        .query_unpaged(statics::INSERT_FRIEND, (user.clone(), friend.clone(), now))
        .await
        .map(|_| ())
        .map_err(From::from);
    println!("Add friend result for {} -> {}: {:?}", user, friend, res);

    Some(res)
}

pub async fn fetch_friends(
    session: &scylla::client::session::Session,
    user: String,
) -> Option<Vec<(String, DateTime<Utc>)>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_FRIENDS, (user,))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    let mut friends = Vec::new();

    for row in (query_rows.rows::<(Option<&str>, Option<DateTime<Utc>>)>()).ok()? {
        if let Ok((Some(friend), Some(created_at))) = row {
            friends.push((friend.to_string(), created_at));
        }
    }

    Some(friends)
}

pub async fn delete_friend(
    session: &scylla::client::session::Session,
    user: String,
    friend: String,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(statics::DELETE_FRIEND, (user, friend))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}
