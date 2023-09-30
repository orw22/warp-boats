use crate::{
    db::SharedConnectionPool,
    errors::Error,
    handlers::helpers::acquire_connection,
    models::boat::{Boat, NewBoat, UpdateBoat},
    schema::boats,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use log::error;
use warp::{http::StatusCode, reject, reply};

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
