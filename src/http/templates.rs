use crate::user::User;
use rocket::Route;
use rocket_dyn_templates::Template;
use serde::Serialize;
use std::ops::Deref;

#[derive(Serialize)]
struct Context {
    is_logged_in: bool,
    name: Option<String>,
}

#[get("/")]
fn index(user: User) -> Template {
    Template::render::<_, Context>("index", user.into())
}

impl From<User> for Context {
    fn from(val: User) -> Self {
        match val.deref() {
            Some(it) => Context {
                is_logged_in: true,
                name: Some(it.name.clone()),
            },
            None => Context {
                is_logged_in: false,
                name: None,
            },
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![index]
}
