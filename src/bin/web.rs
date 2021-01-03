use codeword::web::db::InMemGameDB;
use codeword::web::lobby::GameWrapper;
use warp::Filter;

// TODO: Global Game cache / db
// TODO: Pass game cache / db down to api handlers

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

mod filters {

    use super::handlers;
    use codeword::web::db::InMemGameDB;
    use warp::Filter;

    pub fn app(
        db: InMemGameDB,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        return lobby(db.clone()).or(create_player(db.clone()));
    }

    pub fn lobby(
        db: InMemGameDB,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        return warp::path!("lobby")
            .and(warp::filters::method::post())
            .and(with_db(db))
            .and_then(handlers::create_lobby);
    }

    pub fn create_player(
        db: InMemGameDB,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("lobby" / String / "players")
            .and(warp::path::end())
            .and(warp::filters::method::post())
            .and(warp::body::json())
            .and(with_db(db))
            .and_then(handlers::create_player)
    }

    fn with_db(
        db: InMemGameDB,
    ) -> impl Filter<Extract = (InMemGameDB,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}

mod handlers {

    use crate::GameWrapper;
    use anyhow::Result;
    use codeword::players::{Player, SimplePlayer};
    use codeword::web::db::InMemGameDB;
    use codeword::web::lobby::Lobby;
    use uuid::Uuid;

    pub fn generate_game_id() -> String {
        return Uuid::new_v4().to_string();
    }

    pub async fn create_lobby(db: InMemGameDB) -> Result<impl warp::Reply, warp::Rejection> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        // let game = GameWrapper::new(&words).unwrap();
        let game = match GameWrapper::new(&words) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Error initializing game: {:?}", e);
                return Err(warp::reject::custom(e));
            }
        };
        let game_id = generate_game_id();
        let lobby = Lobby::new(&game_id, &(Vec::new()), game);
        let add_res = db.add_new_lobby(lobby).await;
        match add_res {
            Ok(_) => {
                return Ok(String::from(format!("{}", game_id)));
            }
            Err(e) => {
                println!("Error adding game \"{}\" to DB: {:?}", game_id, e);
                // TODO: return proper error.
                return Err(warp::reject::custom(e));
            }
        };
    }

    pub async fn create_player(
        lobby_id: String,
        player: SimplePlayer,
        db: InMemGameDB,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let lobby_res = db.get_lobby(&lobby_id).await;
        if lobby_res.is_err() {
            return Err(warp::reject::not_found());
        }
        let lobby = lobby_res.unwrap();
        let mut lobby_writer = lobby.write().await;
        lobby_writer.add_player_id(&(player.get_id().to_string()));
        // TODO: Create and store WebAppPlayer in global registry.
        return Ok(String::from("ok"));
    }

    pub fn add_player_to_team() {
        todo!()
        // let player = serde_json::from_str(json_data.as_ref());

        // if player.is_err() {
        //     return String::from("Error: Invalid request");
        // }
        // let player = player.unwrap();
        // match arc_game {
        //     Ok(game) => {
        //         println!("Creating player for: {}", lobby_id);
        //         // println!("Creating player: {}", json_data);
        //         match game.write() {
        //             Ok(mut g) => {
        //                 match &mut *g {
        //                     GameWrapper::InitialGame(g0) => {
        //                         g0.add_player_team_one(player);
        //                     }
        //                     GameWrapper::InProgressGame(g0) => {
        //                         g0.add_player_team_one(player);
        //                     }
        //                 };
        //                 return String::from("Ok");
        //             }
        //             Err(_e) => {
        //                 return String::from("Error adding player");
        //             }
        //         }
        //     }
        //     Err(_e) => {
        //         return String::from("Error adding player");
        //     }
        // }
    }
}
