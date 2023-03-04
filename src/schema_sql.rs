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
        points->Float,
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
        id -> BigInt,
        email -> Text,
        password -> Text,
        is_admin -> Bool,
        points -> BigInt,
    }
}
