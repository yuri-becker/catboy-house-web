use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

const TOML: &str = include_str!("../Services.toml");

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    Private,
    Admin
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    pub name: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub group: Option<String>,
    pub category: Option<Category>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Services {
    services: HashMap<String, Service>,
}

impl Services {
    pub fn parse() -> Self {
        toml::from_str::<Services>(TOML).unwrap()
    }
}

impl Deref for Services {
    type Target = HashMap<String, Service>;
    fn deref(&self) -> &Self::Target {
        &self.services
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for Services {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(
            request
                .rocket()
                .state::<Services>()
                .expect("Services should be in Rocket state")
                .clone(),
        )
    }
}
