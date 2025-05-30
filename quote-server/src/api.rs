use crate::*;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "quote-server", description = "Quote API")
    )
)]
pub struct ApiDoc;

pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new()
        .routes(routes!(get_quote))
        .routes(routes!(get_tagged_quote))
        .routes(routes!(get_random_quote))
}

async fn get_quote_by_id(db: &SqlitePool, quote_id: &str) -> Result<response::Response, http::StatusCode> {
    let quote_result = quote::get(db, quote_id).await;
    match quote_result {
        Ok((quote, tags)) => Ok(JsonQuote::new(quote, tags).into_response()),
        Err(e) => {
            log::warn!("quote fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}