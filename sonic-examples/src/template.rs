use askama_actix::{Template, TemplateToResponse};

#[derive(Template)]
#[template(path = "search.html")]

pub struct Search<'a> {
    pub name: &'a str,
}
