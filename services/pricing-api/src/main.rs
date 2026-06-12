mod db;
mod dispatch;
mod handlers;
mod models;
mod routes;

use axum::Router;

#[tokio::main]
async fn main() {
    let pool = db::try_connect().await;
    if pool.is_some() {
        println!("Connected to database");
    }
    let state = db::AppState { db: pool };
    let app: Router = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!(
        "Pricing API listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}
