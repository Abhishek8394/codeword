use codeword::web::db::InMemGameDB;
use codeword::web::app::filters;
use warp::Filter;

#[tokio::main]
async fn main() {
    println!("Web App!");
    let db = InMemGameDB::new();
    let api = filters::app(db.clone());
    let routes = api.with(warp::log("codeword"));
    let port = 8080;
    println!("Starting on port: {}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
