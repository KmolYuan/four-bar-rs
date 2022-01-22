use crate::update::extract;
use actix_files::Files;
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{
    cookie::{time::Duration, Cookie, SameSite},
    post,
    web::{Data, Json, Path},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use four_bar_ui::{dump_csv, parse_csv, sha512, LoginInfo};
use std::{
    collections::BTreeMap,
    env::current_dir,
    fs::{read_to_string, write},
    io::Result,
    ops::Deref,
    slice::from_ref,
};
use temp_dir::TempDir;

const COOKIE_LIFE: Duration = Duration::weeks(1);

// Usernames
struct Users(BTreeMap<String, String>);

impl Deref for Users {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[post("/login/{user}")]
async fn login(
    users: Data<Users>,
    user: Path<String>,
    id: Identity,
    json: Json<LoginInfo>,
) -> impl Responder {
    let user = user.into_inner();
    match users.get(&user) {
        Some(pwd) if user == json.account && pwd == &json.password => {
            id.remember(user.clone());
            let cookie = Cookie::build("username", user)
                .same_site(SameSite::Lax)
                .max_age(COOKIE_LIFE)
                .path("/")
                .finish();
            HttpResponse::Ok().cookie(cookie).finish()
        }
        _ => HttpResponse::Forbidden().finish(),
    }
}

#[post("/logout")]
async fn logout(id: Identity, req: HttpRequest) -> impl Responder {
    id.forget();
    let mut builder = HttpResponse::Ok();
    if let Some(mut cookie) = req.cookie("username") {
        cookie.make_removal();
        builder.cookie(cookie);
    }
    builder.finish()
}

pub(crate) fn serve(port: u16) -> Result<()> {
    let users = Data::new(users()?);
    let temp = TempDir::new()?;
    extract(temp.path())?;
    let path = temp.path().to_path_buf();
    println!("Serve at: http://localhost:{}/", port);
    println!("Unpacked archive at: {:?}", &path);
    println!("Press Ctrl+C to close the server...");
    let server = HttpServer::new(move || {
        let cookie_policy = CookieIdentityPolicy::new(&[0; 32])
            .name("auth-cookie")
            .max_age(COOKIE_LIFE)
            .secure(true);
        App::new()
            .wrap(IdentityService::new(cookie_policy))
            .app_data(users.clone())
            .service(login)
            .service(logout)
            .service(Files::new("/", &path).index_file("index.html"))
    })
    .bind(("localhost", port))?
    .run();
    actix_web::rt::System::new().block_on(server)
}

fn users() -> Result<Users> {
    let users = current_dir()?.join("users.csv");
    let mut map = BTreeMap::new();
    if users.is_file() {
        for user in parse_csv::<LoginInfo>(&read_to_string(users)?).unwrap() {
            map.insert(user.account, user.password);
        }
    } else {
        let mut user = LoginInfo::default();
        user.password = sha512(&user.password);
        write(&users, dump_csv(from_ref(&user)).unwrap())?;
        map.insert(user.account, user.password);
    }
    Ok(Users(map))
}
