use rocket::{Rocket, State};
use rocket::fairing::{AdHoc, Fairing, Info, Kind};
use rocket::response::NamedFile;
use std::path::PathBuf;

pub struct ResourcesFairing;
impl Fairing for ResourcesFairing {
    fn info(&self) -> Info {
        Info {
            name: "Resources route fairing.",
            kind: Kind::Attach,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        Ok(rocket
            .mount("/resources", routes![files])
            .attach(AdHoc::on_attach(|rocket| {
                let root = PathBuf::from(::expect_config(rocket.config(), "resource_dir"));
                Ok(rocket.manage(Root(root)))
            })))
    }
}

pub fn fairing() -> ResourcesFairing {
    ResourcesFairing
}

struct Root(PathBuf);

#[get("/<file..>")]
fn files(file: PathBuf, root: State<Root>) -> Option<NamedFile> {
    NamedFile::open(root.0.join(file)).ok()
}
