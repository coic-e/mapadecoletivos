// @generated automatically by Diesel CLI.

diesel::table! {
    images (id) {
        id -> Int4,
        path -> Varchar,
        organization_id -> Int4,
    }
}

diesel::table! {
    organizations (id) {
        id -> Int4,
        name -> Varchar,
        latitude -> Numeric,
        longitude -> Numeric,
        #[sql_name = "type"]
        type_ -> Varchar,
        city -> Varchar,
        uf -> Varchar,
        email -> Varchar,
        social -> Varchar,
        about -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::joinable!(images -> organizations (organization_id));

diesel::allow_tables_to_appear_in_same_query!(
    images,
    organizations,
);
