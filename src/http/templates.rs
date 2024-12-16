use crate::http::user::User;
use crate::services::Services;
use rocket::Route;
use rocket_dyn_templates::{context, Template};
use std::ops::Deref;

struct UserContext {
    is_logged_in: bool,
    name: Option<String>,
}

#[get("/")]
fn index(user: User, services: Services) -> Template {
    let user_context: UserContext = (&user).into();
    let (mut private_services, mut public_services): (Vec<_>, Vec<_>) = services
        .iter()
        .filter(|(_, service)| match user.deref().clone() {
            Some(user) => match service.group.clone() {
                None => true,
                Some(group) => user.groups.contains(&group),
            },
            None => service.group.is_none(),
        })
        .map(|(_, service)| service)
        .partition(|it| it.group.is_some());
    private_services.sort_by(|a, b| a.name.cmp(&b.name));
    public_services.sort_by(|a, b| a.name.cmp(&b.name));
    Template::render(
        "index",
        context! {
            public_services,
            private_services,
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
    routes![index]
}
