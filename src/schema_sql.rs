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
    }
}

table! {
    markers(id) {
        id -> BigInt,
        point-> postgis_diesel::sql_types::Geometry,
        creation_date->Timestamptz,
        created_by->BigInt,
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
    }
}
