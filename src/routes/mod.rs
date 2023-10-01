pub mod boat;
pub mod filters;
pub mod jwt;
pub mod user;

use crate::{db::SharedConnectionPool, rate_limiting::KeyedRateLimiter, routes};
use warp::Filter;

pub fn all_routes(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    routes::boat::routes(pool.clone(), rate_limiter)
        .or(routes::user::routes(pool))
        .or(routes::jwt::routes())
}
