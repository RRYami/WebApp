mod handlers;
mod models;
mod routes;

use axum::Router;

#[tokio::main]
async fn main() {
    let app: Router = routes::create_router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Pricing API listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
