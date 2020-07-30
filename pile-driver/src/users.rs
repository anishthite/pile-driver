use diesel;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use serde::{ Serialize, Deserialize };
use diesel::{ QueryId, Queryable, Insertable, AsChangeset};

use crate::schema::{users, invite_tokens};

#[table_name = "users"]
#[derive(Clone, Serialize, Deserialize, QueryId, Queryable, Insertable, AsChangeset, Debug, PartialEq, Eq)]
pub struct User {
    pub email: String, 
    pub username: String,
    pub password: String,
    pub about: Option<String>,
    pub admin: Option<i32>,
    pub timecreated: i64,
    pub parent: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PubUser {
    pub username: String,
    pub timecreated: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UserNode {
    pub user: PubUser,
    pub descendents: Vec<UserNode>
}


impl User{
    pub fn create(user: User, connection: &MysqlConnection) -> User {
        diesel::insert_into(users::table)
            .values(&user)
            .execute(connection)
            .expect("Error creating new user");

        users::table.order(users::username.desc()).first(connection).unwrap()
    }

    pub fn read(connection: &MysqlConnection) -> Vec<User> {
        users::table.order(users::username.asc()).load::<User>(connection).unwrap()
    }

     pub fn read_single(username: String, connection: &MysqlConnection) -> Result<User, diesel::result::Error> {
        users::table.find(username).first(connection)
    }

    pub fn update(username: String, user: User, connection: &MysqlConnection) -> bool {
        diesel::update(users::table.find(username)).set(&user).execute(connection).is_ok()
    }
    pub fn render_single(rootuser: User, connection: &MysqlConnection) -> UserNode {
       let child_users = users::table.filter(users::parent.eq(&rootuser.username)).load::<User>(connection).unwrap();
       let mut descendents: Vec<UserNode> = Vec::new();
       for child_user in &child_users{
           let child_node = User::render_single(User::clone(child_user), &connection);
           descendents.push(child_node);
       }
       let root_node = UserNode {user: PubUser{username: rootuser.username, timecreated: rootuser.timecreated}, descendents: descendents};
       root_node
    }

}


#[table_name = "invite_tokens"]
#[derive(Serialize, Deserialize, QueryId, Queryable, Insertable,AsChangeset, Debug, PartialEq, Eq)]
pub struct InviteToken {
    pub token: String,
    pub creator: String
}
impl InviteToken{
    pub fn create(invitetoken: InviteToken, connection: &MysqlConnection) -> InviteToken {
     diesel::insert_into(invite_tokens::table)
              .values(&invitetoken)
              .execute(connection)
              .expect("Error creating new user");
          
          invite_tokens::table.order(invite_tokens::token.desc()).first(connection).unwrap()
    }
    pub fn read(connection: &MysqlConnection) -> Vec<InviteToken> {
        invite_tokens::table.order(invite_tokens::token.asc()).load::<InviteToken>(connection).unwrap()
    }

   pub fn read_single(token: String, connection: &MysqlConnection) -> Result<InviteToken, diesel::result::Error> {
        invite_tokens::table.find(token).first(connection)
    }
    pub fn delete(invitetoken: String, connection: &MysqlConnection) -> bool {
        diesel::delete(invite_tokens::table.find(invitetoken)).execute(connection).is_ok()
    }
}

