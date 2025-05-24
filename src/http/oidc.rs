use crate::config::Config;
use crate::http::oidc::OidcError::OidcEndpointUnreachable;
use crate::oidc_client::OidcClient;
use openid::{Options, Token};
use rand::{rng, Rng};
use rocket::http::private::cookie::CookieBuilder;
use rocket::http::{CookieJar, SameSite, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::Redirect;
use rocket::{get, Request, State};
use std::convert::Into;
use rand::distr::Alphanumeric;
use OidcError::Unauthorized;

#[derive(Responder)]
enum OidcError {
    #[response(status = 401)]
    Unauthorized(()),
    #[response(status = 500)]
    Misconfiguration(()),
    #[response(status = 500)]
    OidcEndpointUnreachable(()),
}

#[derive(Debug)]
struct InvalidCallbackError {}

impl From<InvalidCallbackError> for Outcome<CallbackQuery, InvalidCallbackError> {
    fn from(val: InvalidCallbackError) -> Self {
        Outcome::Error((Status::BadRequest, val))
    }
}
#[derive(Debug)]
struct CallbackError {
    error: String,
    error_description: String,
    iss: String,
}

struct CallbackSuccess {
    code: String,
}

enum CallbackQuery {
    Error(CallbackError),
    Success(CallbackSuccess),
}

#[async_trait]
impl<'r> FromRequest<'r> for CallbackQuery {
    type Error = InvalidCallbackError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, InvalidCallbackError> {
        let error = request.query_value("error");
        if error.is_some() {
            let Some(Ok(error)) = error else {
                return InvalidCallbackError {}.into();
            };
            let Some(Ok(error_description)) = request.query_value::<String>("error_description")
            else {
                return InvalidCallbackError {}.into();
            };
            let Some(Ok(iss)) = request.query_value::<String>("iss") else {
                return InvalidCallbackError {}.into();
            };
            Outcome::Success(CallbackQuery::Error(CallbackError {
                error,
                error_description,
                iss,
            }))
        } else {
            let Some(Ok(code)) = request.query_value::<String>("code") else {
                return InvalidCallbackError {}.into();
            };
            Outcome::Success(CallbackQuery::Success(CallbackSuccess { code }))
        }
    }
}

#[get("/callback")]
async fn authorize<'r>(
    callback_query: CallbackQuery,
    oidc_client: &State<OidcClient>,
    cookie_jar: &'r CookieJar<'_>,
) -> Result<Redirect, OidcError> {
    match callback_query {
        CallbackQuery::Error(error) => {
            if error.error == "access_denied" {
                return Ok(Redirect::to("/"));
            }
            warn!(
                "Could not log in via OIDC:\n{}\n{}\nISS: {}",
                error.error, error.error_description, error.iss
            );
            Err(OidcError::Misconfiguration(()))
        }
        CallbackQuery::Success(success) => {
            let bearer = oidc_client
                .request_token(&success.code)
                .await
                .map_err(|_| OidcEndpointUnreachable(()))?;
            let mut token = Token::from(bearer.clone());
            let id_token = token.id_token.as_mut().ok_or(OidcEndpointUnreachable(()))?;
            oidc_client
                .decode_token(id_token)
                .map_err(|_| Unauthorized(()))?;
            oidc_client
                .validate_token(id_token, None, None)
                .map_err(|_| Unauthorized(()))?;
            cookie_jar.add_private(
                CookieBuilder::new(
                    "token",
                    bearer.id_token.expect("Bearer should have id token"),
                )
                .same_site(SameSite::Lax)
                .build(),
            );
            Ok(Redirect::to("/"))
        }
    }
}

#[get("/login")]
fn login(oidc_client: &State<OidcClient>) -> Result<Redirect, Status> {
    let state = String::from_utf8(
        rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .collect::<Vec<u8>>(),
    )
    .unwrap();
    let nonce = String::from_utf8(
        rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .collect::<Vec<u8>>(),
    )
    .unwrap();

    let options = Options {
        scope: Some(String::from("openid profile groups")),
        state: Some(state),
        nonce: Some(nonce),
        ..Options::default()
    };
    let url = oidc_client.auth_url(&options);
    info!("Redirecting to OIDC endpoint: {}", url);
    Ok(Redirect::found::<String>(url.into()))
}

#[get("/logout")]
fn logout(
    cookie_jar: &CookieJar<'_>,
    oidc_client: &State<OidcClient>,
    config: &State<Config>,
) -> Redirect {
    cookie_jar.remove_private("user");
    if let Some(id_token) = cookie_jar.get_private("token") {
        let id_token = id_token.value();
        cookie_jar.remove_private("token");
        if let Some(session_endpoint) = &oidc_client.config().end_session_endpoint {
            let mut endpoint = session_endpoint.clone();
            endpoint
                .query_pairs_mut()
                .append_pair("id_token_hint", id_token)
                .append_pair(
                    "post_logout_redirect_uri",
                    (config.host.clone() + "/").as_str(),
                );
            info!("Redirecting to OIDC logout endpoint: {}", endpoint);
            return Redirect::found::<String>(endpoint.to_string());
        }
    }
    Redirect::to("/")
}

pub fn routes() -> Vec<rocket::Route> {
    routes![authorize, login, logout]
}
