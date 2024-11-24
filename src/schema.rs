// @generated automatically by Diesel CLI.

diesel::table! {
    admin_users (id) {
        id -> Integer,
        email -> Text,
        password -> Text,
    }
}

diesel::table! {
    blog_categories (blog_id, category_id) {
        blog_id -> Nullable<Integer>,
        category_id -> Nullable<Integer>,
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
    categories (id) {
        id -> Integer,
        name -> Text,
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
        order -> Integer,
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
    sessions (id) {
        id -> Nullable<Integer>,
        session_id -> Text,
        user_id -> Text,
        data -> Nullable<Text>,
        expires_at -> Timestamp,
    }
}

diesel::table! {
    social_links (id) {
        id -> Nullable<Integer>,
        social_media -> Text,
        social_link -> Text,
    }
}

diesel::joinable!(blog_categories -> blogs (blog_id));
diesel::joinable!(blog_categories -> categories (category_id));

diesel::allow_tables_to_appear_in_same_query!(
    admin_users,
    blog_categories,
    blogs,
    categories,
    experiences,
    messages,
    sessions,
    social_links,
);
