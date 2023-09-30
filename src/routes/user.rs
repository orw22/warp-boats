use std::collections::HashMap;

use crate::{db::SharedConnectionPool, handlers, routes::filters::with_db};
use warp::Filter;

pub fn routes(
    pool: SharedConnectionPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    create_user(pool.clone()).or(add_credit(pool))
}

fn create_user(
    pool: SharedConnectionPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(pool))
        .and_then(handlers::user::create_user)
}

fn add_credit(
    pool: SharedConnectionPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users" / "credit")
        .and(warp::put())
        .and(warp::query::<HashMap<String, String>>())
        .and_then(move |params: HashMap<String, String>| {
            let pool = pool.clone();
            async move {
                handlers::user::add_credit(params.get("amount"), params.get("api_key"), pool).await
            }
        })
}
