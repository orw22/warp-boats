use crate::{consts::BEARER, db::SharedConnectionPool, errors::Error};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    SqliteConnection,
};
use warp::{
    http::header::{HeaderMap, HeaderValue, AUTHORIZATION},
    reject,
};

pub fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, Error> {
    let auth_header = std::str::from_utf8(
        headers
            .get(AUTHORIZATION)
            .ok_or_else(|| Error::InvalidAuthHeader)?
            .as_bytes(),
    )
    .map_err(|_| Error::InvalidAuthHeader)?;
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}

pub async fn acquire_connection(
    pool: &SharedConnectionPool,
) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, warp::Rejection> {
    Ok(pool
        .lock()
        .await
        .acquire()
        .map_err(|_| reject::custom(Error::ConnectionFailed))?)
}
