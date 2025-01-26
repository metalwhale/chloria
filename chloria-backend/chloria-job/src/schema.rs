// @generated automatically by Diesel CLI.

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
