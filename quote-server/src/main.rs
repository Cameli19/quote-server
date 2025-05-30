mod error;
mod quote;
mod templates;
mod web;
mod api;

use error::*;
use quote::*;
use templates::*;

extern crate log;
extern crate mime;

use axum::{
    self,
    extract::{Path, Query, State, Json},
    http,
    response::{self, IntoResponse},
    routing,
};
use clap::Parser;
extern crate fastrand;
use serde::{Serialize, Deserialize};
use sqlx::{Row, SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use std::borrow::Cow;
use std::sync::Arc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

struct AppState {
    db: SqlitePool,
    current_quote: Quote,
}

fn get_db_uri(db_uri: Option<&str>) -> Cow<str> {
    if let Some(db_uri) = db_uri {
        db_uri.into()
    } else if let Ok(db_uri) = std::env::var("DATABASE_URL") {
        db_uri.into()
    } else {
        "sqlite://db/quote.db".into()
    }
}

fn extract_db_dir(db_uri: &str) -> Result<&str, QuoteError> {
    if db_uri.starts_with("sqlite://") && db_uri.ends_with(".db") {
        let start = db_uri.find(':').unwrap() + 3;
        let mut path = &db_uri[start..];
        if let Some(end) = path.rfind('/') {
            path = &path[..end];
        } else {
            path = "";
        }
        Ok(path)
    } else {
        Err(QuoteError::InvalidDbUri(db_uri.to_string()))
    }
}
async fn get_quote() -> response::Html<String> {
    let quote = IndexTemplate::quote(&THE_QUOTE);
    response::Html(quote.to_string())
}


async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let app = axum::Router::new()
        .route("/", routing::get(get_quote))
        .route_service(
            "/quote.css",
            services::ServeFile::new_with_mime("assets/static/quote.css", &mime::TEXT_CSS)
        );
    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("quote serve: error: {}", err);
        std::process::exit(1);
    }
}
