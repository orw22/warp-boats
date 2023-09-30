mod config;
mod consts;
mod credit;
mod db;
mod errors;
mod handlers;
mod models;
mod rate_limiting;
mod responses;
mod routes;
mod schema;

use std::num::NonZeroU32;
use std::sync::Arc;

use crate::{
    config::get_config,
    db::{ConnectionPool, SharedConnectionPool},
    errors::handle_rejection,
    rate_limiting::QUOTA_PER_SECOND,
};
use anyhow::Result;
use governor::{Quota, RateLimiter};
use log::LevelFilter;
use tokio::sync::Mutex;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<()> {
    let config = get_config();
    let port = config.get_int("server.port")? as u16;
    let pool: SharedConnectionPool = Arc::new(Mutex::new(ConnectionPool::new(
        &config.get_string("database.url")?,
    )));
    let rate_limiter = Arc::new(Mutex::new(RateLimiter::keyed(Quota::per_second(
        NonZeroU32::new(QUOTA_PER_SECOND).unwrap(),
    ))));

    // set up logger
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();

    // serve API
    warp::serve(
        routes::boat::routes(pool.clone(), rate_limiter)
            .or(routes::jwt::routes())
            .or(routes::user::routes(pool))
            .with(warp::cors().allow_any_origin().allow_credentials(true))
            .recover(handle_rejection),
    )
    .run(([127, 0, 0, 1], port))
    .await;

    Ok(())
}
