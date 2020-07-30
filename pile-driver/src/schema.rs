table! {
    chunks (chunk_id) {
        chunk_id -> Varchar,
        server_id -> Text,
        time_started -> Bigint,
        complete -> Tinyint,
    }
}
