
table! {
    pills (id) {
        id -> Int4,
        text -> Text,
        author -> Text,
        source -> Text,
    }
}

table! {
    trash_types (id){
        id->Integer,
        name->Text,
    }
}

table! {
    markers(id) {
        id -> Integer,
        point-> postgis_diesel::sql_types::Geometry,
        creation_date->Timestamptz,
        created_by->Integer,
        trash_type_id -> Integer,
    }
}

table! {
    marker_images(id){
        id -> Integer,
        path -> Text,
        refers_to -> Integer,
    }
}
table! {
    users(id){
        id -> Integer,
        email -> Text,
        password -> Text,
        is_admin -> Bool,
    }
}