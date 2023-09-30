use crate::{errors::Error, schema::users};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};

pub async fn deduct_credit(
    api_key: &String,
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<(), Error> {
    let credit: i32 = users::table
        .select(users::credit)
        .filter(users::api_key.eq(api_key))
        .first(conn)
        .map_err(|_| Error::NotFound)?;

    if credit > 0 {
        diesel::update(users::table.filter(users::api_key.eq(api_key)))
            .set(users::credit.eq(users::credit - 1))
            .execute(conn)
            .map_err(|_| Error::NotFound)?;
        Ok(())
    } else {
        Err(Error::NoCredit)
    }
}
