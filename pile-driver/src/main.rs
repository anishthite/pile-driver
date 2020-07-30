#![feature(plugin)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate diesel;
extern crate rocket_cors;
extern crate time;
use rocket::http::Method; 

use rocket_cors::{
    AllowedHeaders, AllowedOrigins, 
    Cors, CorsOptions 
};

extern crate r2d2;
extern crate r2d2_diesel;
extern crate bcrypt;

use bcrypt::{hash, verify};
use serde::{ Serialize, Deserialize };
use rocket_contrib::json::{Json, JsonValue};
use rocket::http::{ Status, Cookie, Cookies };
use rocket::Response;
use std::io::Cursor;
extern crate nanoid;
use nanoid::nanoid;
mod users;
mod items;
mod schema;
mod db;
use users::{User, InviteToken};
use items::{Item};
use chrono::prelude::*;


//CORS
fn make_cors() -> Cors {
     let allowed_origins = AllowedOrigins::all(); // 4.
   //let allowed_origins = AllowedOrigins::some_exact(&[ // 4.
   //    "https:gthackerhome.github.io",
   //    "http://localhost:8080",
   //    "http://127.0.0.1:8080",
   //    "http://localhost:8000",
   //    "http://0.0.0.0:8000"
   //]);

    CorsOptions { // 5.
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(), 
        allowed_headers: AllowedHeaders::all(),
    //    allowed_headers: AllowedHeaders::some(&[
    //        "Authorization",
    //        "Accept",
    //        "Access-Control-Allow-Origin", 
    //    ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}



//Form Structs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]   
pub struct LoginForm {
    pub username: String,
    pub password: String
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]   
pub struct LogoutForm {
    pub username: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCookie {
    pub username: String,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]   
pub struct CreatePostForm {
    pub title: String, 
    pub url: Option<String>,
    pub text: Option<String>,
} 

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]   
pub struct CreateCommentForm {
    pub text: String,
    pub parentid: String

}

//User Functions

#[post("/create_token")]
fn create_token(cookies: Cookies, connection: db::Connection) -> Json<String> {
      let alphabet: [char; 16] = [
          '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f'
      ];
      let myid = nanoid!(10, &alphabet);
  
      let author = match cookies.get("username") {
          Some(x) => String::from(x.value()),
          None => return Json(String::from("Are you signed in?")),
      };
      let newtoken = InviteToken{creator: author, token: myid.clone()};
      InviteToken::create(newtoken, &connection);
      Json(myid)
}

//Create user, 
#[post("/sign_up", data = "<user>")]
fn sign_up(user: Json<User>, connection: db::Connection) -> Status {
    
    let mut insert = User {admin : Some(0), ..user.into_inner()};
    let curr_time = Utc::now().timestamp();
    insert.timecreated = curr_time;
    let hash_result  = hash(insert.password, 4);

    match hash_result {
        Ok(x) => insert.password = x,
        Err(_e) => return Status::NotAcceptable,
    }

    let curr_names = User::read(&connection);

    for name in curr_names.iter(){
        if name.username == insert.username {
            return Status::Conflict ;      
        }
    }
    let tokens = InviteToken::read(&connection);
    if (tokens.len() == 0){
           User::create(insert, &connection);
           return Status::Created;
    }
    
    
    for token in tokens.iter(){
        if token.token == insert.parent {
           InviteToken::delete(token.token.clone(), &connection); 
           insert.parent = token.creator.clone();
           User::create(insert, &connection);
           return Status::Created;
        }
    }
    return Status::NotAcceptable;
}

//Login User
#[post("/login", data = "<form>")]
fn login(form: Json<LoginForm>, connection: db::Connection) -> Response<'static> {
    let my_user = LoginForm{..form.into_inner()};
    let my_user_copy = my_user.username.clone();
    let result = User::read_single(my_user.username, &connection);
    let mut response = Response::new();
    let mybool;
    match result {
        Ok(x) =>  mybool = verify(my_user.password,&x.password),
        Err(_e) => {response.set_raw_status(699, "Tripped a Wire"); return response;}
    }
    match mybool {
        Ok(x) => {
                    if x == true {  
                        let cookie = Cookie::build("username", my_user_copy.clone())
                            .domain("greetez.com")
                            .path("/")
                            .secure(true)
                            .http_only(true)
                            .finish();
                            response.set_header(cookie);
                            response.set_sized_body(Cursor::new(my_user_copy));
                            return response;
                    }
                    else {
                        response.set_raw_status(699, "Tripped a Wire"); return response;
                    }
                }
        Err(_e) => {response.set_raw_status(699, "Tripped a Wire"); return response;}
 
    }

}

//Logout User
#[post("/logout", data = "<form>")]
fn logout(form: Json<LogoutForm>) -> Response<'static> {
    let my_user = LogoutForm{..form.into_inner()};
    let my_user_copy = my_user.username.clone();
    let mut response = Response::new();
    let mut now = time::now();
    now.tm_year -= 1;  
    let mut cookie = Cookie::build("username", my_user_copy)
        .domain("greetez.com")
        .path("/")
        .secure(true)
        .http_only(true)
        .finish();
        cookie.set_expires(now);
        response.set_header(cookie);
        return response;
 

}


//test cookie
//#[get("/cookie")]
//fn cookie() -> Response<'static> {
//    let mut response = Response::new();                    
//    let cookie = Cookie::build("username", "user")
//                            .domain("greetez.com")
//                            .path("/")
//                            .secure(true)
//                            .http_only(true)
//                            .finish();
//                            response.set_header(cookie);
//                            let mycookie = Cookie::build("username", "user")
//                                .domain("gthackerhome.github.io")
//                                .path("/")
//                                .secure(true)
//                                .finish();
//                            response.adjoin_header(mycookie);
//                            return response;
//}
//



//read user
#[get("/<username>")]
fn view(username: String, connection: db::Connection) -> JsonValue {
    format!("username: {}", username);

    let myuser_result = User::read_single(username, &connection);
    match myuser_result {
        Ok(user) => return json!({"username":user.username, "about":user.about, "admin":user.admin, "date created":user.timecreated, "parent":user.parent}),
        Err(_e) => return json!({"failure":"database err"}) 
    }
}
#[get("/usertree")]
fn usertree(connection: db::Connection) -> JsonValue{
    let rootuser = match User::read_single(String::from("aquajet"), &connection){
    Ok(x) => x,
    Err(_e) => {return json!({"failure":"database err"})}
    };
    let usertree = User::render_single(rootuser, &connection);
    return json!(Json(usertree).into_inner());
}





//Post API


//read posts
#[get("/posts")]
fn posts(connection: db::Connection) -> Json<Vec<Item>> {
    let posts = Item::read_posts(&connection);
    return Json(posts); 
}

#[get("/<id>")]
fn render(id: String, connection: db::Connection) -> JsonValue{
    let rootitem = match Item::read_single(id, &connection){
    Ok(x) => x,
    Err(_e) => {return json!({"failure":"database err"})}
    };
    let itemtree = Item::render_single(rootitem, &connection);
    return json!(Json(itemtree).into_inner());
}




//create_post
#[post("/create_post", data = "<item>")]
fn create_post(item: Json<CreatePostForm>, cookies: Cookies, connection: db::Connection) -> Status {
    let alphabet: [char; 16] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f'
    ];
    let my_item_form = CreatePostForm{..item.into_inner()};
    let myid = nanoid!(10, &alphabet); 
    let curr_time = Utc::now().timestamp(); 
    
    let author = match cookies.get("username") {
        Some(x) => String::from(x.value()),
        None => return Status::NotAcceptable,
    };

    let new_item = Item {
        id: myid, 
        parentid: None,
        title: Some(my_item_form.title),
        descendents: Some(0), 
        score: Some(0), 
        time: curr_time,
        author: author,
        itemtype: String::from("post"),
        url: my_item_form.url,
        text: my_item_form.text
    };
    Item::create(new_item, &connection);
    Status::Created

}

//create_comment
#[post("/create_comment", data = "<item>")]
fn create_comment(item: Json<CreateCommentForm>, cookies: Cookies, connection: db::Connection) -> Status {
    let alphabet: [char; 16] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f'
    ];
    let my_item_form = CreateCommentForm{..item.into_inner()};
    let myid = nanoid!(10, &alphabet); 
    let curr_time = Utc::now().timestamp(); 
    
    let author = match cookies.get("username") {
        Some(x) => String::from(x.value()),
        None => return Status::NotAcceptable,
    };

    let new_item = Item {
        id: myid, 
        parentid: Some(my_item_form.parentid),
        title: None,
        descendents: Some(0), 
        score: Some(0), 
        time: curr_time,
        author: author,
        itemtype: String::from("comment"),
        url: None,
        text: Some(my_item_form.text)
    };
    Item::create(new_item, &connection);
    Status::Created

}



//render item


fn main() {
    rocket::ignite()
        .manage(db::connect())
        .mount("/user_api", routes![view, sign_up, login, logout, create_token, usertree])
        .mount("/item_api", routes![render, posts, create_post, create_comment])
        .attach(make_cors())
        .launch();
}
