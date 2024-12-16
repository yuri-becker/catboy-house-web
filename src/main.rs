mod config;
mod http;
mod oidc_client;
mod user;

#[macro_use]
extern crate rocket;

use crate::config::Config;
use crate::http::{oidc, templates};
use oidc_client::OidcClient;
use rocket::config::SecretKey;
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::Template;
use shuttle_runtime::SecretStore;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {
    let config = Config::from(&secrets);
    let rocket = rocket::build()
        .configure(rocket::Config {
            secret_key: SecretKey::from(
                secrets.get("secret").expect("Please set secret").as_bytes(),
            )
            .clone(),
            ..rocket::Config::default()
        })
        .mount("/assets", FileServer::from(relative!("assets")))
        .mount("/oidc", oidc::routes())
        .mount("/", templates::routes())
        .manage(OidcClient::new(&config).await)
        .manage(config)
        .attach(Template::fairing());
    Ok(rocket.into())
}
