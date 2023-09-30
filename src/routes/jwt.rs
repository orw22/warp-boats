use std::collections::HashMap;

use crate::handlers;
use warp::Filter;

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    generate_token().or(decode_token())
}

fn generate_token() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("generate-token")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and_then(|params: HashMap<String, String>| async move {
            handlers::jwt::generate_token(params.get("id"), params.get("role")).await
        })
}

fn decode_token() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("decode-token")
        .and(warp::get())
        .and(warp::header::headers_cloned())
        .and_then(handlers::jwt::decode_token)
}
