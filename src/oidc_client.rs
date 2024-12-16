use crate::config::Config;
use openid::{Client, CompactJson, Discovered, SingleOrMultiple, StandardClaims, Userinfo};
use rocket::serde::{Deserialize, Serialize};
use std::ops::Deref;
use url::Url;

pub struct OidcClient {
    client: Client<Discovered, Claims>,
}

impl Deref for OidcClient {
    type Target = Client<Discovered, Claims>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl OidcClient {
    pub async fn new(config: &Config) -> Self {
        let client = Client::<Discovered, Claims>::discover(
            config.oidc_client_id.clone(),
            config.oidc_client_password.clone(),
            config.host.clone() + "/oidc/callback",
            Url::parse(config.oidc_endpoint.as_str()).expect("oidc_endpoint is not a valid URL"),
        )
        .await;
        if client.is_err() {
            panic!(
                "Failed to discover OIDC provider: {:?}",
                client.unwrap_err()
            );
        }
        let client = client.unwrap();
        info!("Successfully discovered OIDC provider");
        OidcClient { client }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct Claims {
    pub groups: Option<Vec<String>>,
    #[serde(flatten)]
    pub standard_claims: StandardClaims,
}

impl CompactJson for Claims {}

impl openid::Claims for Claims {
    fn iss(&self) -> &Url {
        self.standard_claims.iss()
    }

    fn sub(&self) -> &str {
        self.standard_claims.sub()
    }

    fn aud(&self) -> &SingleOrMultiple<String> {
        self.standard_claims.aud()
    }

    fn exp(&self) -> i64 {
        self.standard_claims.exp()
    }

    fn iat(&self) -> i64 {
        self.standard_claims.iat()
    }

    fn auth_time(&self) -> Option<i64> {
        self.standard_claims.auth_time()
    }

    fn nonce(&self) -> Option<&String> {
        self.standard_claims.nonce()
    }

    fn at_hash(&self) -> Option<&String> {
        self.standard_claims.at_hash()
    }

    fn c_hash(&self) -> Option<&String> {
        self.standard_claims.c_hash()
    }

    fn acr(&self) -> Option<&String> {
        self.standard_claims.acr()
    }

    fn amr(&self) -> Option<&Vec<String>> {
        self.standard_claims.amr()
    }

    fn azp(&self) -> Option<&String> {
        self.standard_claims.azp()
    }

    fn userinfo(&self) -> &Userinfo {
        self.standard_claims.userinfo()
    }

    fn at_hash_to_vec(&self) -> Option<Vec<u8>> {
        self.standard_claims.at_hash_to_vec()
    }

    fn c_hash_to_vec(&self) -> Option<Vec<u8>> {
        self.standard_claims.c_hash_to_vec()
    }
}
