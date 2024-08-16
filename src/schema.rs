// @generated automatically by Diesel CLI.

diesel::table! {
    blogs (id) {
        id -> Nullable<Integer>,
        title -> Text,
        content -> Text,
        image -> Nullable<Text>,
    }
}

diesel::table! {
    experiences (id) {
        id -> Nullable<Integer>,
        company_name -> Text,
        position -> Text,
        start_date -> Text,
        end_date -> Nullable<Text>,
        responsibility -> Nullable<Text>,
        skills -> Nullable<Text>,
    }
}

diesel::table! {
    social_links (id) {
        id -> Nullable<Integer>,
        social_media -> Text,
        social_link -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    blogs,
    experiences,
    social_links,
);
