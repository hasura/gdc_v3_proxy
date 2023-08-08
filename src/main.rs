mod api;
mod config;
mod error;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use std::{error::Error, net::SocketAddr};

use self::routes::*;
use clap::Parser;

#[derive(Parser)]
struct ServerOptions {
    #[arg(long, env, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let options = ServerOptions::parse();

    let router = Router::new()
        .route("/capabilities", get(get_capabilities))
        .route("/schema", get(get_schema))
        .route("/query", post(post_query))
        .route("/mutation", post(post_mutation))
        .route("/raw", post(post_raw))
        .route("/explain", post(post_explain))
        .route("/health", get(get_health));

    let adresss = format!("0.0.0.0:{}", options.port).parse()?;

    println!("Starting server on {}", &adresss);

    axum::Server::bind(&adresss)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
