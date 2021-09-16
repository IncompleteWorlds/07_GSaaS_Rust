table! {
    t_execution_record (execution_id) {
        execution_id -> Integer,
        user_id -> Text,
        module_id -> Integer,
        module_instance_id -> Integer,
        start_time -> Timestamp,
        stop_time -> Timestamp,
        status -> Text,
        complete_flag -> Bool,
        expiration_time -> Timestamp,
    }
}

table! {
    t_http_access (id) {
        id -> Integer,
        request_time -> Timestamp,
        ip_address -> Text,
        hostname -> Nullable<Text>,
        operation -> Text,
    }
}


allow_tables_to_appear_in_same_query!(
    t_execution_record,
    t_http_access,
);
