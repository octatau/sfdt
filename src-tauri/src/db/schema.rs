// @generated automatically by Diesel CLI.

diesel::table! {
    user (id) {
        id -> Text,
        org_id -> Text,
        username -> Text,
        first_name -> Text,
        last_name -> Text,
        alias -> Text,
        base_url -> Text,
        access_token -> Text,
        refresh_token -> Text,
    }
}
