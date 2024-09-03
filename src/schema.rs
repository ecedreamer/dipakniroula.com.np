// @generated automatically by Diesel CLI.

diesel::table! {
    admin_users (id) {
        id -> Integer,
        email -> Text,
        password -> Text,
    }
}

diesel::table! {
    blogs (id) {
        id -> Nullable<Integer>,
        title -> Text,
        content -> Text,
        image -> Nullable<Text>,
        published_date -> Text,
        modified_date -> Nullable<Text>,
        view_count -> Integer,
        is_active -> Integer,
    }
}

diesel::table! {
    experiences (id) {
        id -> Nullable<Integer>,
        company_name -> Text,
        your_position -> Text,
        start_date -> Text,
        end_date -> Nullable<Text>,
        responsibility -> Nullable<Text>,
        skills -> Nullable<Text>,
        company_link -> Text,
    }
}

diesel::table! {
    messages (id) {
        id -> Nullable<Integer>,
        full_name -> Text,
        email -> Text,
        mobile -> Nullable<Text>,
        subject -> Text,
        message -> Text,
        date_sent -> Text,
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
    admin_users,
    blogs,
    experiences,
    messages,
    social_links,
);
