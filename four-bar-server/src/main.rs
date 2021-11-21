use actix_session::{CookieSession, Session};
use actix_web::{
    get,
    http::header,
    web::{Data, Query},
    App, HttpResponse, HttpServer,
};
use clap::{clap_app, AppSettings};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use serde::Deserialize;
use std::error::Error;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v3/token";

#[get("/")]
fn index(session: Session) -> HttpResponse {
    let link = match session.get("login") {
        Ok(Some(true)) => "logout",
        _ => "login",
    };
    let html = format!(
        r#"<html>
        <head><title>OAuth2 Test</title></head>
        <body>
            <a href="/{0}">{0}</a>
        </body>
        </html>"#,
        link
    );
    HttpResponse::Ok().body(html)
}

#[get("/login")]
fn login(client: Data<BasicClient>) -> HttpResponse {
    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, _csrf_state) = &client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/plus.me".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();
    HttpResponse::Found()
        .header(header::LOCATION, authorize_url.to_string())
        .finish()
}

#[get("/logout")]
fn logout(session: Session) -> HttpResponse {
    session.remove("login");
    HttpResponse::Found()
        .header(header::LOCATION, "/".to_string())
        .finish()
}

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
    scope: String,
}

#[get("/auth")]
fn auth(session: Session, client: Data<BasicClient>, params: Query<AuthRequest>) -> HttpResponse {
    let code = AuthorizationCode::new(params.code.clone());
    let state = CsrfToken::new(params.state.clone());
    let _scope = params.scope.clone();

    // Exchange the code with a token.
    let token = &client.exchange_code(code);
    session.set("login", true).unwrap();
    let html = format!(
        r#"<html>
        <head><title>OAuth2 Test</title></head>
        <body>
            Google returned the following state:
            <pre>{}</pre>
            Google returned the following token:
            <pre>{:?}</pre>
        </body>
    </html>"#,
        state.secret(),
        token
    );
    HttpResponse::Ok().body(html)
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = clap_app! {
        ("Four🍀bar server") =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (setting: AppSettings::ArgRequiredElseHelp)
        (@arg CLIENT_ID: --id +takes_value "Google client ID")
        (@arg CLIENT_SECRET: --secret +takes_value "Google client SECRET")
        (@arg PORT: --port +takes_value "localhost port")
    }
    .get_matches();
    let client_id = args
        .value_of("CLIENT_ID")
        .expect("Missing the CLIENT_ID argument");
    let client_secret = args
        .value_of("CLIENT_ID")
        .expect("Missing the CLIENT_SECRET argument");
    let port = args.value_of("PORT").unwrap_or("5000").parse::<u16>()?;
    println!("Serve at: http://localhost:{}/", port);
    let client = Data::new(
        BasicClient::new(
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
            AuthUrl::new(AUTH_URL.to_string())?,
            Some(TokenUrl::new(TOKEN_URL.to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(format!("http://localhost:{}/auth", port))?),
    );
    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .service(index)
            .service(login)
            .service(logout)
            .service(auth)
    })
    .bind(("localhost", port))?
    .run()
    .await
    .map_err(|e| e.into())
}
