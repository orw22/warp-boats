use crate::{
    config::get_config,
    consts::BEARER,
    db::SharedConnectionPool,
    errors::Error,
    models::{Boat, Claims, NewBoat, NewUser, UpdateBoat},
    responses::TokenResponse,
    schema::{boats, users},
};
use chrono::prelude::*;
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use log::{debug, error};
use uuid::Uuid;
use warp::{
    http::{
        header::{HeaderMap, HeaderValue, AUTHORIZATION},
        StatusCode,
    },
    reject, reply,
};

// Boat handlers

pub async fn get_boat(
    id: i32,
    pool: SharedConnectionPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = acquire_connection(&pool).await?;
    let boat: Boat = boats::table
        .find(&id)
        .first(&mut conn)
        .map_err(|_| reject::custom(Error::NotFound))?;
    Ok(reply::json(&boat))
}

pub async fn get_all_boats(
    pool: SharedConnectionPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = acquire_connection(&pool).await?;
    let boats: Vec<Boat> = boats::table
        .load(&mut conn)
        .map_err(|e| error!("{}", e.to_string()))
        .map_err(|_| reject::custom(Error::NotFound))?;
    Ok(reply::json(&boats))
}

pub async fn create_boat(
    boat: NewBoat,
    pool: SharedConnectionPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = acquire_connection(&pool).await?;
    diesel::insert_into(boats::table)
        .values(&boat)
        .execute(&mut conn)
        .map_err(|_| reject::custom(Error::NotFound))?;
    Ok(reply::with_status(
        reply::json(&String::from("Boat created")),
        StatusCode::CREATED,
    ))
}

pub async fn update_boat(
    id: i32,
    boat: UpdateBoat,
    pool: SharedConnectionPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = acquire_connection(&pool).await?;
    diesel::update(boats::table.filter(boats::id.eq(id)))
        .set((
            boat.name.map(|v| boats::name.eq(v)),
            boat.make.map(|v| boats::make.eq(v)),
            boat.model.map(|v| boats::model.eq(v)),
            boat.year.map(|v| boats::year.eq(v)),
            boat.length.map(|v| boats::length.eq(v)),
            boat.beam.map(|v| boats::beam.eq(v)),
            boat.is_available.map(|v| boats::is_available.eq(v)),
        ))
        .execute(&mut conn)
        .map_err(|_| reject::custom(Error::NotFound))?;
    Ok(reply::json(&format!("Boat {} updated", id)))
}

pub async fn delete_boat(
    id: i32,
    pool: SharedConnectionPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = acquire_connection(&pool).await?;
    diesel::delete(boats::table.filter(boats::id.eq(id)))
        .execute(&mut conn)
        .map_err(|_| reject::custom(Error::NotFound))?;
    Ok(reply::with_status(reply::reply(), StatusCode::NO_CONTENT))
}

// User handlers

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

//  JWT handlers

pub async fn generate_token(
    id: Option<&String>,
    role: Option<&String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let (id, role) = (
        id.ok_or(reject::custom(Error::InvalidParameter))?,
        role.ok_or(reject::custom(Error::InvalidParameter))?,
    );
    let jwt_secret = get_config()
        .get_string("jwt.secret")
        .map_err(|_| reject::custom(Error::JWTCreationFailed))?;
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(12))
        .unwrap()
        .timestamp();
    let claims = Claims {
        sub: id.to_string(),
        role: role.to_string(),
        exp: expiration as usize,
        iat: Utc::now().timestamp() as usize,
    };
    let token = encode(
        &Header::new(Algorithm::HS512),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| reject::custom(Error::JWTCreationFailed))?;

    Ok(reply::json(&TokenResponse { token }))
}

pub async fn decode_token(
    headers: HeaderMap<HeaderValue>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let jwt_secret = get_config()
        .get_string("jwt.secret")
        .map_err(|_| reject::custom(Error::JWTCreationFailed))?;
    let jwt = jwt_from_header(&headers).map_err(|e| reject::custom(e))?;
    let decoded = decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    )
    .map_err(|_| reject::custom(Error::InvalidCredentials))?;

    if &decoded.claims.role != "admin" {
        return Err(reject::custom(Error::NoPermission));
    }
    Ok(reply::json(&decoded.claims))
}

// Helper funcs

fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, Error> {
    let auth_header = std::str::from_utf8(
        headers
            .get(AUTHORIZATION)
            .ok_or_else(|| Error::InvalidAuthHeader)?
            .as_bytes(),
    )
    .map_err(|_| Error::InvalidAuthHeader)?;
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}

async fn acquire_connection(
    pool: &SharedConnectionPool,
) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, warp::Rejection> {
    Ok(pool
        .lock()
        .await
        .acquire()
        .map_err(|_| reject::custom(Error::ConnectionFailed))?)
}
