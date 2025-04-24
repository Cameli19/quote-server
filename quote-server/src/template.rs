use crate::*;

use askama::Template;

#[derive(Template)]
#[Template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub quote: &'a Quote,
}