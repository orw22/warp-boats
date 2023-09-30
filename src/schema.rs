diesel::table! {
    boats (id) {
        id -> Integer,
        name -> Text,
        make -> Text,
        model -> Text,
        year -> Integer,
        length -> Nullable<Float>,
        beam -> Nullable<Float>,
        is_available -> Integer,
    }
}

diesel::table! {
    users (email) {
        email -> Text,
        api_key -> Text,
        credit -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(boats, users,);
