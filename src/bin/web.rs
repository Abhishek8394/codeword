use anyhow::{bail, Result};
use codeword::game::Game;
use codeword::game::InProgressGame;
use codeword::game::InitialGame;
use codeword::players::Player;
use codeword::players::SimplePlayer;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use warp::Filter;
use codeword::web::OnlinePlayer;

use std::sync::{Arc, RwLock};
// TODO: Global Game cache / db
// TODO: Pass game cache / db down to api handlers

pub trait WebAppPlayer: Player + Clone + DeserializeOwned {}

impl WebAppPlayer for SimplePlayer {}

pub enum GameWrapper<P: WebAppPlayer> {
    InitialGame(Game<InitialGame, P>),
    InProgressGame(Game<InProgressGame, P>),
}
// impl <S, P: Player>GameWrapper<P>{
//     pub fn get_game(self) -> Game<S, P>{
//         match self {
//             GameWrapper::InitialGame(g) => g,
//             GameWrapper::InProgressGame(g) => g,
//         }
//     }
// }

type ArcGameWrapper<P> = Arc<RwLock<GameWrapper<P>>>;

pub fn get_arc_game_wrapper<P: WebAppPlayer>(game: GameWrapper<P>) -> ArcGameWrapper<P> {
    Arc::new(RwLock::new(game))
}

#[derive(Clone)]
pub struct GameDB<P: WebAppPlayer> {
    db: Arc<RwLock<HashMap<String, ArcGameWrapper<P>>>>,
}

impl<P: WebAppPlayer> GameDB<P> {
    pub fn new() -> Self {
        GameDB {
            db: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_num_games(&self) -> Result<usize> {
        Ok(self.db.read().unwrap().len())
    }

    pub fn add_new_game(&self, game_id: &str, game: GameWrapper<P>) -> Result<()> {
        eprintln!("Adding game: {}", game_id);
        match self.db.write() {
            Ok(mut db) => {
                db.insert(String::from(game_id), get_arc_game_wrapper(game));
            }
            Err(_e) => {
                bail!("cannot add game now");
            }
        };
        Ok(())
    }

    pub fn get_game(&self, game_id: &str) -> Result<ArcGameWrapper<P>> {
        match self.db.read() {
            Ok(h_map) => match h_map.get(game_id) {
                Some(arc_game) => {
                    return Ok(arc_game.clone());
                }
                None => {
                    bail!(format!("No game found for: {:}", game_id));
                }
            },
            Err(e) => {
                eprintln!("{:?}", e);
                bail!("Error reading DB")
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Web App!");
    let db: GameDB<SimplePlayer> = GameDB::new();
    let api = filters::app(db.clone());
    let routes = api.with(warp::log("codeword"));

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

mod filters {
    use crate::WebAppPlayer;

    use super::handlers;
    use crate::GameDB;
    use warp::Filter;

    pub fn app<P: WebAppPlayer + Send + Sync>(
        db: GameDB<P>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        return lobby(db.clone()).or(create_player(db.clone()));
    }

    pub fn lobby<P: WebAppPlayer + Send + Sync>(
        db: GameDB<P>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("lobby")
            .and(warp::filters::method::post())
            .and(with_db(db))
            .map(handlers::create_lobby)
    }

    pub fn create_player<P: WebAppPlayer + Send + Sync>(
        db: GameDB<P>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("lobby" / String / "players")
            .and(warp::path::end())
            .and(warp::filters::method::post())
            .and(warp::body::json())
            .and(with_db(db))
            .map(handlers::create_player)
    }

    fn with_db<P: WebAppPlayer + Send + Sync>(
        db: GameDB<P>,
    ) -> impl Filter<Extract = (GameDB<P>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}

mod handlers {

    use crate::WebAppPlayer;

    use crate::GameWrapper;

    use crate::GameDB;
    use codeword::game::Game;
    use codeword::game::InitialGame;

    pub fn generate_game_id<P: WebAppPlayer>(db: &GameDB<P>) -> String {
        String::from("game-")
            + &(match db.get_num_games() {
                Ok(n) => format!("{}", n),
                Err(_) => String::from("-err"),
            } as String)
    }

    pub fn create_lobby<P: WebAppPlayer>(db: GameDB<P>) -> String {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let game: Game<InitialGame, P> = match Game::new(&words) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Error initializaing game: {:?}", e);
                return String::from("Couldnt init game;");
            }
        };
        let game_id = generate_game_id(&db);
        match db.add_new_game(&game_id, GameWrapper::InitialGame(game)) {
            Ok(_) => {
                return String::from(format!("{}", game_id));
            }
            Err(e) => {
                println!("Error adding game \"{}\" to DB: {:?}", game_id, e);
                return String::from("Error!");
            }
        };
    }

    pub fn create_player<P: WebAppPlayer>(lobby_id: String, player: P, db: GameDB<P>) -> String {
        let arc_game = db.get_game(&lobby_id);
        // let player = serde_json::from_str(json_data.as_ref());

        // if player.is_err() {
        //     return String::from("Error: Invalid request");
        // }
        // let player = player.unwrap();
        match arc_game {
            Ok(game) => {
                println!("Creating player for: {}", lobby_id);
                // println!("Creating player: {}", json_data);
                match game.write() {
                    Ok(mut g) => {
                        match &mut *g {
                            GameWrapper::InitialGame(g0) => {
                                g0.add_player_team_one(player);
                            }
                            GameWrapper::InProgressGame(g0) => {
                                g0.add_player_team_one(player);
                            }
                        };
                        return String::from("Ok");
                    }
                    Err(_e) => {
                        return String::from("Error adding player");
                    }
                }
            }
            Err(_e) => {
                return String::from("Error adding player");
            }
        }
    }
}
