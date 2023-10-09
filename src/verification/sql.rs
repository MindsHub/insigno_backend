use chrono::Utc;
use diesel::sql_types::BigInt;


sql_function!(fn time_to_verify(user_id: BigInt) -> Timestamptz);

// there is no way to handle multiple values at the moment
// sql_function!(fn get_to_verify(user_id: BigInt) -> ());

table! {
    verification_sessions(id){
        id -> Nullable<BigSerial>,
        user_id -> BigInt,
        completition_date -> Nullable<Timestamptz>,
    }
}

#[derive(Insertable, Queryable, QueryableByName, AsChangeset)]
#[diesel(table_name = verification_sessions)]
pub(crate) struct VerificationSession {
    pub id: Option<i64>,
    pub user_id: i64,
    pub completition_date: Option<chrono::DateTime<Utc>>,
}


table! {
    image_verifications(id){
        id -> Nullable<BigInt>,
        verification_session -> Nullable<BigInt>,
        image_id -> BigInt,
        marker_id -> BigInt,
        verdict-> Nullable<Bool>,
    }
}

#[derive(Insertable, Queryable, QueryableByName, AsChangeset)]
#[diesel(table_name = image_verifications)]
pub struct ImageVerificationDiesel {
    pub id: Option<i64>,
    pub verification_session: Option<i64>,
    pub image_id: i64,
    pub marker_id: i64,
    pub verdict: Option<bool>,
}
