use crate::http::user::User;
use crate::services::{Category, Services};
use rocket::Route;
use rocket_dyn_templates::{context, Template};
use std::ops::Deref;
use rocket::fs::{relative, NamedFile};

struct UserContext {
    is_logged_in: bool,
    name: Option<String>,
}
#[get("/favicon.ico")]
async fn favicon() -> std::io::Result<NamedFile> {
    NamedFile::open(relative!("assets/favicon.ico")).await
}

#[get("/")]
fn index(user: User, services: Services) -> Template {
    let user_context: UserContext = (&user).into();
    let mut services: Vec<_> = services
        .iter()
        .filter(|(_, service)| match user.deref().clone() {
            Some(user) => match service.group.clone() {
                None => true,
                Some(group) => user.groups.contains(&group),
            },
            None => service.group.is_none(),
        })
        .map(|(_, service)| service)
        .collect();

    services.sort_by(|a, b| a.name.cmp(&b.name));
    let public_services: Vec<_> = services.iter().filter(|it| it.category.is_none()).collect();
    let private_services: Vec<_> = services
        .iter()
        .filter(|it| {
            it.category
                .clone()
                .is_some_and(|it| it.eq(&Category::Private))
        })
        .collect();
    let admin_services: Vec<_> = services
        .iter()
        .filter(|it| {
            it.category
                .clone()
                .is_some_and(|it| it.eq(&Category::Admin))
        })
        .collect();
    Template::render(
        "index",
        context! {
            public_services,
            private_services,
            admin_services,
            is_logged_in: user_context.is_logged_in,
            name: user_context.name
        },
    )
}

impl From<&User> for UserContext {
    fn from(val: &User) -> Self {
        match val.deref() {
            Some(it) => UserContext {
                is_logged_in: true,
                name: Some(it.name.clone()),
            },
            None => UserContext {
                is_logged_in: false,
                name: None,
            },
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![favicon, index]
}
