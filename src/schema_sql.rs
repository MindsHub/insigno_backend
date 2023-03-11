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
        id -> BigInt,
        point-> postgis_diesel::sql_types::Geometry,
        creation_date->Timestamptz,
        resolution_date->Nullable<Timestamptz>,
        created_by->BigInt,
        solved_by->Nullable<BigInt>,
        marker_types_id -> BigInt,
    }
}

table! {
    marker_images(id){
        id -> BigInt,
        path -> Text,
        refers_to -> BigInt,
    }
}

table! {
    users(id){
        id -> Nullable<BigInt>,
        name -> Text,
        password -> Text,
        is_admin -> Bool,
        points -> Double,
    }
}

table! {
    user_sessions(user_id){
        user_id -> BigInt,
        token -> Text,
        refresh_date -> Timestamptz,
    }
}
/*
table!{
    groups(id){
        id->BigInt,
        points->Double,
        creation_date->Timestamptz,
        end_date->Nullable<Timestamptz>,
    }
}*/
