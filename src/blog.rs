use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use pulldown_cmark::{html, Parser};
use rocket::{Rocket, State};
use rocket::fairing::{AdHoc, Fairing, Info, Kind};
use rocket_contrib::Template;
use toml;

/// Installation.
/// Add a "blog_dir" key to Rocket.toml
pub struct BlogFairing;
impl Fairing for BlogFairing {
    fn info(&self) -> Info {
        Info {
            name: "Blog route fairing.",
            kind: Kind::Attach,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        Ok(rocket
            .mount("/blog", routes![index, entry])
            .attach(AdHoc::on_attach(|rocket| {
                let index =
                    read_blog_index(Path::new(::expect_config(rocket.config(), "blog_dir")));
                Ok(rocket.manage(index.expect("unable to read Index.toml")))
            })))
    }
}

pub fn fairing() -> BlogFairing {
    BlogFairing
}

/// Indexing
///
#[derive(Serialize)]
struct IndexContext<'a> {
    parent: &'static str,
    items: &'a BlogIndex,
}

#[get("/index")]
fn index(blog_index: State<BlogIndex>) -> Template {
    let context = IndexContext {
        parent: "base",
        items: &blog_index,
    };

    Template::render("blog/index", &context)
}

type BlogIndex = Vec<BlogEntry>;

#[derive(Serialize)]
struct BlogEntry {
    key: String,
    path: PathBuf,
    title: String,
}

fn read_blog_index(blogdata: &Path) -> io::Result<BlogIndex> {
    #[derive(Debug, Deserialize)]
    struct Item {
        title: String,
        filename: String,
    }

    #[derive(Debug, Deserialize)]
    struct RawIndex {
        entries: Vec<Item>,
    }

    let mut toml_buf = String::new();
    fs::File::open(blogdata.join("Index.toml"))?.read_to_string(&mut toml_buf)?;

    toml::from_str::<RawIndex>(&toml_buf)
        .map(|index| {
            index
                .entries
                .iter()
                .map(|item| BlogEntry {
                    key: strip_extension(Path::new(&item.filename)).into(),
                    path: blogdata.join(item.filename.clone()),
                    title: item.title.clone(),
                })
                .collect()
        })
        .map_err(|toml_err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Error parsing index: {}", toml_err),
            )
        })
}

fn strip_extension(path: &Path) -> &str {
    path.file_stem().unwrap().to_str().unwrap()
}

/// Individual entries.
///
/// TODO(zcd): better than O(N) lookup if I ever write that much stuff
/// TODO(zcd): figure out how to surface errors as Rocket NotFound

#[get("/<entry_key>")]
fn entry(entry_key: String, blog_index: State<BlogIndex>) -> Result<Template, io::Error> {
    let rendered_md = blog_index
        .iter()
        .find(|entry| entry.key == entry_key)
        .map(|entry| render_md_file(&entry.path))
        .unwrap_or(Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Bad entry: {}", entry_key),
        )))?;

    Ok(Template::render(
        "blog/entry",
        &[("contents", rendered_md), ("parent", "base".into())]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
    ))
}

fn render_md_file(path: &Path) -> io::Result<String> {
    let mut entry = fs::File::open(path)?;

    let mut contents = String::new();
    entry.read_to_string(&mut contents)?;

    let parser = Parser::new(&contents);
    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);
    Ok(html_buf)
}
