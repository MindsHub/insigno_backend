use chrono::Utc;
use diesel::sql_types::BigInt;

/*
CREATE TABLE IF NOT EXISTS public.verification_sessions
(
    id BIGSERIAL NOT NULL,
    user_id BIGINT NOT NULL,
    completition_date timestamp with time zone,

    CONSTRAINT verification_sessions_id PRIMARY KEY (id),
    CONSTRAINT user_id_fkey FOREIGN KEY (user_id)
        REFERENCES public.users (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION
);*/
sql_function!(fn time_to_verify(user_id: BigInt)-> Timestamptz);
sql_function!(fn can_verify(user_id: BigInt)-> Bool);
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

/*
CREATE TABLE IF NOT EXISTS public.image_verifications
(
    id BIGSERIAL NOT NULL,
    verification_session BIGINT,
    image_id BIGINT NOT NULL,
    verdict BOOLEAN,

    CONSTRAINT image_verifications_id PRIMARY KEY (id),
    CONSTRAINT verification_session_id_fkey FOREIGN KEY (verification_session)
        REFERENCES public.verification_sessions (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE NO ACTION,
    CONSTRAINT image_id_fkey FOREIGN KEY (image_id)
        REFERENCES public.marker_images (id) MATCH SIMPLE
        ON UPDATE cascade
        ON DELETE cascade
); */
table! {
    image_verification(id){
        id -> Nullable<BigInt>,
        verification_session -> Nullable<BigInt>,
        image_id -> BigInt,
        verdict-> Nullable<Bool>,
    }
}

#[derive(Insertable, Queryable, QueryableByName, AsChangeset)]
#[diesel(table_name = image_verification)]
pub(crate) struct ImageVerification {
    pub id: Option<i64>,
    pub verification_session: Option<i64>,
    pub image_id: i64,
    pub verdict: Option<bool>,
}
