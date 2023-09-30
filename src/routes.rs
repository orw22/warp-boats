use std::collections::HashMap;

use crate::{
    credit::deduct_credit, db::SharedConnectionPool, errors::Error, handlers,
    rate_limiting::KeyedRateLimiter,
};
use warp::{reject, Filter};

pub fn all_routes(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_all_boats(pool.clone(), rate_limiter.clone())
        .or(create_boat(pool.clone(), rate_limiter.clone()))
        .or(get_boat(pool.clone(), rate_limiter.clone()))
        .or(update_boat(pool.clone(), rate_limiter.clone()))
        .or(delete_boat(pool.clone(), rate_limiter))
        .or(create_user(pool.clone()))
        .or(add_credit(pool))
        .or(generate_token())
        .or(decode_token())
}

// Boat routes

fn get_boat(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // note: use path! macro instead of path() function
    warp::path!("boats" / i32)
        .and(warp::get())
        .and(process_api_key(pool.clone(), rate_limiter))
        .and(with_db(pool))
        .and_then(handlers::get_boat)
}

fn get_all_boats(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("boats")
        .and(warp::get())
        .and(process_api_key(pool.clone(), rate_limiter))
        .and(with_db(pool))
        .and_then(handlers::get_all_boats)
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
        .and_then(handlers::create_boat)
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
        .and_then(handlers::update_boat)
}

fn delete_boat(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("boats" / i32)
        .and(warp::delete())
        .and(process_api_key(pool.clone(), rate_limiter))
        .and(with_db(pool))
        .and_then(handlers::delete_boat)
}

// User routes

fn create_user(
    pool: SharedConnectionPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("users")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(pool))
        .and_then(handlers::create_user)
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
                handlers::add_credit(params.get("amount"), params.get("api_key"), pool).await
            }
        })
}

// JWT routes

fn generate_token() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("generate-token")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and_then(|params: HashMap<String, String>| async move {
            handlers::generate_token(params.get("id"), params.get("role")).await
        })
}

fn decode_token() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("decode-token")
        .and(warp::get())
        .and(warp::header::headers_cloned())
        .and_then(handlers::decode_token)
}

// helpers

fn with_db(
    pool: SharedConnectionPool,
) -> impl Filter<Extract = (SharedConnectionPool,), Error = std::convert::Infallible> + Clone {
    // adds database connection pool to the request for use in handlers
    warp::any().map(move || pool.clone())
}

fn process_api_key(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    // User API key processing: extracts API key from query params, checks with rate limiter, deducts credit
    warp::query::<HashMap<String, String>>()
        .and_then(move |params: HashMap<String, String>| {
            let rate_limiter = rate_limiter.clone();
            let pool = pool.clone();
            async move {
                match params.get("api_key") {
                    Some(api_key) => {
                        let rate_limiter = rate_limiter.lock().await;
                        // check rate limiter (blocking)
                        if rate_limiter.check_key(api_key).is_err() {
                            return Err(reject::custom(Error::RateLimitExceeded));
                        }

                        let mut conn = pool
                            .lock()
                            .await
                            .acquire()
                            .map_err(|_| reject::custom(Error::ConnectionFailed))?;

                        // deduct credit if rate limit not exceeded
                        deduct_credit(&api_key, &mut conn)
                            .await
                            .map_err(|e| reject::custom(e))?;

                        Ok(())
                    }
                    None => Err(reject::custom(Error::MissingAPIKey)),
                }
            }
        })
        .untuple_one() // filter doesn't extract anything so api_key param is not expected from handlers
}
