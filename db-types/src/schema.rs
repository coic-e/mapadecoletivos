// @generated automatically by Diesel CLI.

diesel::table! {
    admins (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    images (id) {
        id -> Int4,
        path -> Varchar,
        organization_id -> Int4,
        position -> Int4,
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
        about -> Varchar,
        created_at -> Timestamp,
        status -> Varchar,
        rejection_reason -> Nullable<Text>,
        reviewed_at -> Nullable<Timestamp>,
        reviewed_by -> Nullable<Int4>,
        // Ajuste manual sobre o print-schema: o Diesel assume que todo array
        // do Postgres pode ter elemento nulo, o que tornaria isto
        // Vec<Option<String>> no modelo. Só a API escreve nesta coluna e ela
        // nunca grava nulo, então Array<Text> vale e evita o Option em cada
        // gênero. Reaplique este ajuste se regenerar o arquivo.
        genres -> Array<Text>,
        address -> Nullable<Varchar>,
        instagram -> Nullable<Varchar>,
        soundcloud -> Nullable<Varchar>,
        bandcamp -> Nullable<Varchar>,
        youtube -> Nullable<Varchar>,
        spotify -> Nullable<Varchar>,
        website -> Nullable<Varchar>,
        is_active -> Bool,
        frequency -> Nullable<Varchar>,
        slug -> Varchar,
    }
}

diesel::joinable!(images -> organizations (organization_id));
diesel::joinable!(organizations -> admins (reviewed_by));

diesel::allow_tables_to_appear_in_same_query!(admins, images, organizations,);
