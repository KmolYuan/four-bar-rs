use crate::update::extract;
use actix_files::Files;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{
    middleware::Logger,
    post,
    web::{Data, Json, Path},
    App, HttpResponse, HttpServer, Responder,
};
use four_bar_ui::{dump_csv, parse_csv, sha512, LoginInfo};
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

#[post("/login/{user}")]
async fn login(
    users: Data<Users>,
    user: Path<String>,
    id: Identity,
    json: Json<LoginInfo>,
) -> impl Responder {
    match users.0.get(user.as_ref()) {
        Some(pwd) if sha512(&user) == json.account && sha512(pwd) == json.password => {
            id.remember(user.into_inner());
            HttpResponse::Ok()
        }
        _ => HttpResponse::Forbidden(),
    }
}

#[post("/logout")]
async fn logout(id: Identity) -> impl Responder {
    id.forget();
    HttpResponse::Ok()
}

pub(crate) async fn serve(port: u16) -> Result<()> {
    let users = Data::new(users()?);
    let temp = TempDir::new()?;
    extract(temp.path()).await?;
    let path = temp.path().to_path_buf();
    println!("Serve at: http://localhost:{}/", port);
    println!("Unpacked archive at: {:?}", &path);
    println!("Press Ctrl+C to close the server...");
    HttpServer::new(move || {
        App::new()
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .secure(true),
            ))
            .wrap(Logger::default())
            .app_data(users.clone())
            .service(login)
            .service(logout)
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
        for user in parse_csv::<LoginInfo>(&read_to_string(users)?).unwrap() {
            map.insert(user.account, user.password);
        }
    } else {
        let user = LoginInfo::default();
        write(&users, dump_csv(from_ref(&user)).unwrap())?;
        map.insert(user.account, user.password);
    }
    Ok(Users(map))
}
