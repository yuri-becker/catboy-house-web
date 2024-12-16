use crate::oidc_client::{Claims, OidcClient};
use log::{debug, warn};
use openid::{Empty, Jws};
use rocket::request::{FromRequest, Outcome};
use rocket::{async_trait, Request};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct User(Option<LoggedInUser>);
#[derive(Debug, Clone)]
pub struct LoggedInUser {
    pub name: String,
    pub groups: Vec<String>,
}

impl Deref for User {
    type Target = Option<LoggedInUser>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(User::from(from_req(request)))
    }
}
fn from_req(request: &Request<'_>) -> Option<LoggedInUser> {
    let oidc_client = request
        .rocket()
        .state::<OidcClient>()
        .expect("OIDC_Client is not in State, this should not occur.");
    let token = request.cookies().get_private("token");
    let mut token = Jws::<Claims, Empty>::new_encoded(token?.value());
    oidc_client
        .decode_token::<Claims>(&mut token)
        .inspect_err(|err| {
            warn!("Could not decode token: {}", err);
        })
        .ok()?;
    oidc_client
        .validate_token(&token, None, None)
        .inspect_err(|err| {
            debug!("Failed to validate token: {}", err);
        })
        .ok()?;
    let payload = token
        .payload()
        .inspect_err(|_| {
            debug!("Token does not have payload");
        })
        .ok()?;
    Some(payload.clone().into())
}

impl From<Claims> for LoggedInUser {
    fn from(value: Claims) -> Self {
        info!("value: {:?}", value);
        let standard_claims = value.standard_claims;
        let userinfo = standard_claims.userinfo;
        Self {
            name: userinfo
                .name
                .or(userinfo.preferred_username)
                .unwrap_or(standard_claims.sub),
            groups: value.groups.unwrap_or_default(),
        }
    }
}

impl From<Option<LoggedInUser>> for User {
    fn from(value: Option<LoggedInUser>) -> Self {
        User(value)
    }
}
