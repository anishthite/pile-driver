create table chunks (
`chunk_id`  VARCHAR(200) PRIMARY KEY NOT NULL,
`server_id` TEXT NOT NULL,
`time_started` BIGINT NOT NULL,
`complete` TINYINT NOT NULL
);
