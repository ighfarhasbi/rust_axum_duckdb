mod db;
mod handlers;
mod models;
mod router;
mod response_model;
mod user_handler;

#[tokio::main]
async fn main() {
    let conn = db::initialize_db().unwrap();
    let app = router::create_router(conn);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(); // port server aplikasi
    axum::serve(listener, app).await.unwrap(); // run server aplikasi dan db
}
