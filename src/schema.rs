// @generated automatically by Diesel CLI.

diesel::table! {
    pills (id) {
        id -> Int4,
        text -> Text,
        author -> Text,
        source -> Text,
    }
}
