use crate::{
    db::SharedConnectionPool, errors::Error, handlers::helpers::acquire_connection,
    models::user::NewUser, schema::users,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use log::debug;
use uuid::Uuid;
use warp::{http::StatusCode, reject, reply};

pub async fn create_user(
    user: NewUser,
    pool: SharedConnectionPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("Creating user...");
    let mut conn = acquire_connection(&pool).await?;
    let api_key = Uuid::new_v4();
    diesel::insert_into(users::table)
        .values(&NewUser {
            email: user.email,
            api_key: Some(api_key.to_string()),
        })
        .execute(&mut conn)
        .map_err(|_| reject::custom(Error::ConnectionFailed))?;
    Ok(reply::with_status(
        reply::json(&format!("User created. Your API key is {}", &api_key)),
        StatusCode::CREATED,
    ))
}

pub async fn add_credit(
    amount: Option<&String>,
    api_key: Option<&String>,
    pool: SharedConnectionPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let amount = amount.ok_or(reject::custom(Error::InvalidParameter))?;
    let api_key = api_key.ok_or(reject::custom(Error::InvalidParameter))?;
    let mut conn = acquire_connection(&pool).await?;
    let amount = amount
        .parse::<i32>()
        .map_err(|_| reject::custom(Error::InvalidParameter))?;
    diesel::update(users::table.filter(users::api_key.eq(api_key)))
        .set(users::credit.eq(users::credit + amount))
        .execute(&mut conn)
        .map_err(|_| reject::custom(Error::NotFound))?;
    Ok(reply::with_status(
        reply::json(&String::from("Added credit")),
        StatusCode::CREATED,
    ))
}
