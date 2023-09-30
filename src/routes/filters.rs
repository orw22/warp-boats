use std::collections::HashMap;

use crate::{
    credit::deduct_credit, db::SharedConnectionPool, errors::Error, rate_limiting::KeyedRateLimiter,
};
use warp::{reject, Filter};

pub fn with_db(
    pool: SharedConnectionPool,
) -> impl Filter<Extract = (SharedConnectionPool,), Error = std::convert::Infallible> + Clone {
    // adds database connection pool to the request for use in handlers
    warp::any().map(move || pool.clone())
}

pub fn process_api_key(
    pool: SharedConnectionPool,
    rate_limiter: KeyedRateLimiter,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    // API key processing: extracts API key from query params, checks rate limiter, deducts credit
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
        .untuple_one() // filter doesn't extract anything so that api_key param is not expected from handlers
}
