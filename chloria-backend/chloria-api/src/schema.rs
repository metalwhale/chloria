// @generated automatically by Diesel CLI.

diesel::table! {
    client_authentications (id) {
        id -> Int4,
        method -> Text,
        identifier -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    client_credentials (id) {
        id -> Int4,
        api_key -> Text,
        api_secret -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    clients (id) {
        id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    news (id) {
        id -> Int4,
        source_name -> Text,
        article_id -> Text,
        link -> Nullable<Text>,
        title -> Nullable<Text>,
        short_text -> Nullable<Text>,
        long_text -> Nullable<Text>,
        image_path -> Nullable<Text>,
        published_time -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(client_authentications -> clients (id));
diesel::joinable!(client_credentials -> clients (id));

diesel::allow_tables_to_appear_in_same_query!(client_authentications, client_credentials, clients, news,);
