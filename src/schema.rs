// @generated automatically by Diesel CLI.

diesel::table! {
    admin_users (id) {
        id -> Integer,
        email -> Text,
        password -> Text,
    }
}

diesel::table! {
    app_settings (key) {
        key -> Text,
        value -> Text,
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

diesel::table! {
    questions (id) {
        id -> Integer,
        topic -> Text,
        difficulty -> Text,
        question_text -> Text,
        options -> Jsonb,
        correct_answer -> Text,
    }
}

diesel::table! {
    quiz_sessions (id) {
        id -> Integer,
        session_uuid -> Text,
        questions_json -> Jsonb,
        created_at -> Timestamp,
    }
}

diesel::table! {
    quiz_attempts (id) {

        id -> Nullable<Integer>,
        player_name -> Text,
        player_email -> Text,
        topic -> Nullable<Text>,
        difficulty -> Text,
        num_questions -> Integer,
        score -> Integer,
        total_questions -> Integer,
        answers_json -> Nullable<Jsonb>,
        played_at -> Timestamp,
    }
}

diesel::joinable!(blog_categories -> blogs (blog_id));
diesel::joinable!(blog_categories -> categories (category_id));

diesel::allow_tables_to_appear_in_same_query!(
    admin_users,
    app_settings,
    blog_categories,
    blogs,
    categories,
    experiences,
    messages,
    questions,
    quiz_attempts,
    quiz_sessions,
    sessions,
    social_links,
);
