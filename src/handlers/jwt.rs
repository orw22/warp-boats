use crate::{
    config::get_config, errors::Error, handlers::helpers::jwt_from_header, models::jwt::Claims,
    responses::TokenResponse,
};
use chrono::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use warp::{
    http::header::{HeaderMap, HeaderValue},
    reject, reply,
};

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
