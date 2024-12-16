use shuttle_runtime::SecretStore;

pub struct Config {
    pub oidc_endpoint: String,
    pub oidc_client_id: String,
    pub oidc_client_password: String,
    pub host: String,
}

impl From<&SecretStore> for Config {
    fn from(value: &SecretStore) -> Self {
        Config {
            oidc_endpoint: value
                .get("oidc_endpoint")
                .expect("Please set oidc_endpoint"),
            oidc_client_id: value
                .get("oidc_client_id")
                .expect("Please set oidc_client_id"),
            oidc_client_password: value
                .get("oidc_client_password")
                .expect("Please set oidc_client_password"),
            host: value.get("host").expect("Please set host"),
        }
    }
}
