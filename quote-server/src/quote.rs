use crate::*;

use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;

use crate::QuoteError;

use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JsonQuote {
    id: String,
    content: String, 
    author: String,
    tags: HashSet<String>,
    source: String,
}

#[derive(Clone)]
pub struct Quote {
    pub id: String,
    pub content: String,
    pub author: String,
    pub source: String,
}

pub fn read_quotes<P: AsRef<Path>>(quotes_path: P) -> Result<Vec<JsonQuote>, QuoteError> {
    let f = std::fs::File::open(quotes_path.as_ref())?;
    let quotes = serde_json::from_reader(f)?;
    Ok(quotes)
}

impl JsonQuote {
    pub fn new(quote: Quote, tags: Vec<String>) -> Self {
        let tags = tags.into_iter().collect();
        Self {
            id: quote.id,
            content: quote.content,
            author: quote.author,
            tags,
            source: quote.source,
        }
    }

    pub fn to_quote(&self) -> (Quote, impl Iterator<Item = &str>) {
        let quote = Quote {
            id: self.id.clone(),
            content: self.content.clone(),
            author: self.author.clone(),
            source: self.source.clone(),
        };
        let tags = self.tags.iter().map(String::deref);
        (quote, tags)
    }
}

impl axum::response::IntoResponse for &JsonQuote {
    fn into_response(self) -> axum::response::Response {
        (http::StatusCode::OK, axum::Json(&self)).into_response()
    }
}

pub async fn get(db: &SqlitePool, quote_id: &str) -> Result<(Quote, Vec<String>), sqlx::Error> {
    let quote = sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE id = $1;", quote_id)
        .fetch_one(db)
        .await?;

    let tags: Vec<String> = sqlx::query_scalar!("SELECT tag FROM tags WHERE quote_id = $1;", quote_id)
        .fetch_all(db)
        .await?;

    Ok((quote, tags))
}

pub async fn get_tagged<'a, I>(db: &SqlitePool, tags: I) -> Result<Option<String>, sqlx::Error>
    where I: Iterator<Item=&'a str>
{
    let mut qtx = db.begin().await?;
    sqlx::query("DROP TABLE IF EXISTS qtags;").execute(&mut *qtx).await?;
    sqlx::query("CREATE TEMPORARY TABLE qtags (tag VARCHR(200));")
        .execute(&mut *qtx)
        .await?;
    for tag in tags {
        sqlx::query("INSERT INTO qtags VALUES ($1);")
            .bind(tag)
            .execute(&mut *qtx)
            .await?;
    }
    let quote_ids = sqlx::query("SELECT DISTINCT quote_id FROM tags JOIN qtags ON tags.tag = qtags.tag ORDER BY RANDOM() LIMIT 1;")
        .fetch_all(&mut *qtx)
        .await?;
    let nquote_ids = quote_ids.len();
    let result = if nquote_ids == 1 {
        Some(quote_ids[0].get(0))
    } else {
        None
    };
    qtx.commit().await?;

    Ok(result)
}

pub async fn get_random(db: &SqlitePool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!("SELECT id FROM quotes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await
}

pub async fn add(db: &SqlitePool, quote: JsonQuote) -> Result<(), sqlx::Error> {
    let mut qtx = db.begin().await?;

    sqlx::query!(
        r#"INSERT INTO quotes
        (id, content, author, source)
        VALUES ($1, $2, $3, $4);"#,
        quote.id,
        quote.content,
        quote.author,
        quote.source,
    )
    .execute(&mut *qtx)
    .await?;

    for tag in quote.tags {
        sqlx::query!(
            r#"INSERT INTO tags (quote_id, tag) VALUES ($1, $2);"#,
            quote.id,
            tag,
        )
            .execute(&mut *qtx)
            .await?;
    }

    qtx.commit().await?;
    Ok(())
}