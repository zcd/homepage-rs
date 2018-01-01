#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate pulldown_cmark;
extern crate rocket;
extern crate rocket_contrib;
extern crate toml;

#[macro_use]
extern crate serde_derive;

mod blog;
mod resources;

fn expect_config<'a>(config: &'a rocket::Config, key: &str) -> &'a str {
    config
        .get_str(key)
        .expect(format!("{:?} not found in Rocket.toml", key).as_str())
}

mod root {
    use rocket::response::Redirect;
    use rocket_contrib::Template;
    use std::collections::HashMap;

    #[get("/")]
    fn home() -> Template {
        ::rocket_contrib::Template::render(
            "home",
            &[("parent", "base")]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
        )
    }

    #[get("/favicon.ico")]
    fn favicon() -> Redirect {
        Redirect::permanent("/resources/favicon.ico")
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![root::home, root::favicon])
        .attach(blog::fairing())
        .attach(resources::fairing())
        .attach(::rocket_contrib::Template::fairing())
        .launch();
}
