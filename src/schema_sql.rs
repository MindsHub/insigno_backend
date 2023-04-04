table! {
    pills (id) {
        id -> BigInt,
        text -> Text,
        author -> Text,
        source -> Text,
        accepted -> Bool,
    }
}

table! {
    marker_types (id){
        id->BigInt,
        name->Text,
        points->Double,
    }
}

table! {
    markers(id) {
        id -> Nullable<BigInt>,
        point-> postgis_diesel::sql_types::Geometry,
        creation_date->Nullable<Timestamptz>,
        resolution_date->Nullable<Timestamptz>,
        created_by->BigInt,
        solved_by->Nullable<BigInt>,
        marker_types_id -> BigInt,
    }
}

table! {
    user_sessions(user_id){
        user_id -> BigInt,
        token -> Text,
        refresh_date -> Timestamptz,
    }
}
