table! {
    t_http_access (id) {
        id -> Integer,
        request_time -> Timestamp,
        ip_address -> Text,
        hostname -> Nullable<Text>,
        operation -> Text,
    }
}

table! {
    t_license (id) {
        id -> Text,
        user_id -> Text,
        license -> Text,
        created -> Timestamp,
        expire_at -> Timestamp,
    }
}

table! {
    t_user (id) {
        id -> Text,
        username -> Text,
        password -> Text,
        email -> Text,
        license_id -> Text,
        created -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    t_http_access,
    t_license,
    t_user,
);
