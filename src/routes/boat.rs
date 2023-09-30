use crate::{
    db::SharedConnectionPool,
    handlers,
    rate_limiting::KeyedRateLimiter,
    routes::filters::{process_api_key, with_db},
};
use warp::Filter;

pub fn routes(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_all_boats(pool.clone(), rate_limiter.clone())
        .or(create_boat(pool.clone(), rate_limiter.clone()))
        .or(get_boat(pool.clone(), rate_limiter.clone()))
        .or(update_boat(pool.clone(), rate_limiter.clone()))
        .or(delete_boat(pool, rate_limiter))
}

fn get_boat(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // note: use path! macro instead of path() function
    warp::path!("boats" / i32)
        .and(warp::get())
        .and(process_api_key(pool.clone(), rate_limiter))
        .and(with_db(pool))
        .and_then(handlers::boat::get_boat)
}

fn get_all_boats(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("boats")
        .and(warp::get())
        .and(process_api_key(pool.clone(), rate_limiter))
        .and(with_db(pool))
        .and_then(handlers::boat::get_all_boats)
}

fn create_boat(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("boats")
        .and(warp::post())
        .and(warp::body::json())
        .and(process_api_key(pool.clone(), rate_limiter))
        .and(with_db(pool))
        .and_then(handlers::boat::create_boat)
}

fn update_boat(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("boats" / i32)
        .and(warp::put())
        .and(warp::body::json())
        .and(process_api_key(pool.clone(), rate_limiter))
        .and(with_db(pool))
        .and_then(handlers::boat::update_boat)
}

fn delete_boat(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("boats" / i32)
        .and(warp::delete())
        .and(process_api_key(pool.clone(), rate_limiter))
        .and(with_db(pool))
        .and_then(handlers::boat::delete_boat)
}
