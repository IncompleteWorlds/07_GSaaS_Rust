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
    t_execution_record,
    t_user,
);