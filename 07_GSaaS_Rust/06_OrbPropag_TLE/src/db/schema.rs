table! {
    t_license (id) {
        id -> Text,
        license -> Text,
        created -> Text,
        expire_at -> Text,
    }
}

table! {
    t_mission (id) {
        id -> Text,
        name -> Text,
        description -> Text,
        created -> Text,
    }
}

table! {
    t_satellite (id) {
        id -> Text,
        mission_id -> Text,
        name -> Text,
        description -> Text,
        launch_date -> Nullable<Text>,
        created -> Text,
    }
}

table! {
    t_ground_station (id) {
        id -> Text,
        name -> Text,
        owner -> Text,
        //created -> Text,
        created -> Timestamp,
    }
}

table! {
    t_antenna (id) {
        id -> Text,
        name -> Text,
        station_id -> Text,
        latitude -> Double,
        longitude -> Double,
        altitude -> Double,
        //created -> Text,
        created -> Timestamp,
    }
}




table! {
    t_user (id) {
        id -> Text,
        username -> Text,
        password -> Text,
        email -> Text,
        license_id -> Text,
        created -> Text,
        logged -> Integer,
        role_id -> Text,
    }
}



joinable!(t_satellite -> t_mission (mission_id));
joinable!(t_antenna   -> t_ground_station (station_id));


allow_tables_to_appear_in_same_query!(
    t_license,
    t_mission,
    t_satellite,
    t_user,

    t_ground_station,
    t_antenna,
);
