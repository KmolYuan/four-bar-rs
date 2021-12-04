use super::update::extract;
use actix_files::{Files, NamedFile};
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{get, post, web::Data, App, HttpResponse, HttpServer, Responder};
use std::{
    io::{Error, Result},
    path::PathBuf,
};
use temp_dir::TempDir;

// Store the index path
struct IndexPath(PathBuf);

#[get("/")]
async fn index(id: Identity, index: Data<IndexPath>) -> Result<NamedFile> {
    if let Some(id) = id.identity() {
        println!("login: {}", id);
    } else {
        println!("not login");
    }
    NamedFile::open(&index.0)
}

#[post("/login")]
async fn login(id: Identity) -> impl Responder {
    id.remember("logged".to_string());
    HttpResponse::Ok()
}

#[post("/logout")]
async fn logout(id: Identity) -> impl Responder {
    id.forget();
    HttpResponse::Ok()
}

pub async fn serve(port: u16) -> Result<()> {
    let temp = TempDir::new().map_err(|e| Error::new(e.kind(), e.to_string()))?;
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
            .app_data(Data::new(IndexPath(path.join("index.html"))))
            .service(index)
            .service(login)
            .service(logout)
            .service(Files::new("/", &path))
    })
    .bind(("localhost", port))?
    .run()
    .await
}
