use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/hello", get(hello_handler));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello_handler() -> &'static str {
    "Hello dca api!"
}
