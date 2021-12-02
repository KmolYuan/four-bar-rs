use super::update::extract;
use actix_files::Files;
use actix_web::{get, App, HttpResponse, HttpServer};
use std::io::{Error, Result};
use temp_dir::TempDir;

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/index.html"))
        .finish()
}

pub async fn serve(port: u16) -> Result<()> {
    let temp = TempDir::new().map_err(|e| Error::new(e.kind(), e.to_string()))?;
    extract(temp.path()).await?;
    let path = temp.path().to_path_buf();
    println!("Serve at: http://localhost:{}/", port);
    println!("Global archive at: {:?}", &path);
    println!("Press Ctrl+C to close the server...");
    HttpServer::new(move || App::new().service(index).service(Files::new("/", &path)))
        .bind(("localhost", port))?
        .run()
        .await
}
