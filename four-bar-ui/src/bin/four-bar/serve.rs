use crate::update::extract;
use actix_files::Files;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{
    post,
    web::{Data, Json},
    App, HttpResponse, HttpServer, Responder,
};
use four_bar_ui::{read_csv, sha512, write_csv, LoginInfo};
use std::{
    collections::BTreeMap,
    env::current_dir,
    fs::{read_to_string, write},
    io::Result,
    slice::from_ref,
};
use temp_dir::TempDir;

// Usernames
struct Users(BTreeMap<String, String>);

#[post("/login")]
async fn login(users: Data<Users>, id: Identity, json: Json<LoginInfo>) -> impl Responder {
    match users.0.get(&json.account) {
        Some(pwd) if sha512(pwd) == json.password => {
            id.remember(json.account.clone());
            HttpResponse::Ok()
        }
        _ => HttpResponse::Forbidden(),
    }
}

pub async fn serve(port: u16) -> Result<()> {
    let users = Data::new(users()?);
    let temp = TempDir::new()?;
    extract(temp.path()).await?;
    let path = temp.path().to_path_buf();
    println!("Serve at: http://localhost:{}/", port);
    println!("Global archive at: {:?}", &path);
    println!("Press Ctrl+C to close the server...");
    HttpServer::new(move || {
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .secure(true),
            ))
            .app_data(users.clone())
            .service(login)
            .service(Files::new("/", &path).index_file("index.html"))
    })
    .bind(("localhost", port))?
    .run()
    .await
}

fn users() -> Result<Users> {
    let users = current_dir()?.join("users.csv");
    let mut map = BTreeMap::new();
    if users.is_file() {
        for user in read_csv::<LoginInfo>(&read_to_string(users)?).unwrap() {
            map.insert(user.account, user.password);
        }
    } else {
        let user = LoginInfo::default();
        write(&users, write_csv(from_ref(&user)).unwrap())?;
        map.insert(user.account, user.password);
    }
    Ok(Users(map))
}
