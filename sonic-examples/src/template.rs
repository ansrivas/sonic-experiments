use askama_actix::Template;

#[derive(Template)]
#[template(path = "search.html")]

pub struct Search;
