use diesel;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use serde::{ Serialize, Deserialize };
use diesel::{ QueryId, Queryable, Insertable, AsChangeset};
use crate::schema::{chunks};

#[table_name = "chunks"]
#[derive(Clone, Serialize, Deserialize, QueryId, Queryable, Insertable, AsChangeset, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub chunk_id: String,
    pub server_id: String,
    pub time_started: i64,
    pub complete: bool,
}    
    
impl Chunk{
    pub fn create(chunk: Chunk, connection: &MysqlConnection) -> Chunk {
        diesel::insert_into(chunks::table)
            .values(&item)
            .execute(connection)
            .expect("Error creating new item");
       chunks::table.order(chunks::chunk_id.desc()).first(connection).unwrap()
    }
   
    //get chunks that have not been complete
    pub fn read_chunks(connection: &MysqlConnection) -> Vec<Chunk> {
        items::table.filter(items::itemtype.eq("post")).order(items::time.desc()).load::<Item>(connection).unwrap()
    }

    pub fn read(connection: &MysqlConnection) -> Vec<Item> {
        items::table.order(items::id.desc()).load::<Item>(connection).unwrap()
    }
    
    pub fn read_single(id: String, connection: &MysqlConnection) -> Result<Item, diesel::result::Error> {
        items::table.find(id).first(connection)
    }

    pub fn update(id: String, item: Item, connection: &MysqlConnection) -> bool {
        diesel::update(items::table.find(id)).set(&item).execute(connection).is_ok()
    }
    pub fn delete(id: String, connection: &MysqlConnection) -> bool {
        diesel::delete(items::table.find(id)).execute(connection).is_ok()
    }
}
