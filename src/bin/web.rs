
use codeword::players::SimplePlayer;
use std::collections::HashMap;
use codeword::game::InProgressGame;
use codeword::game::InitialGame;
use codeword::players::Player;
use codeword::game::Game;
use warp::Filter;
use anyhow::{Result, bail};

use std::sync::{Arc, RwLock};
// TODO: Global Game cache / db
// TODO: Pass game cache / db down to api handlers

pub enum GameWrapper<P: Player>{
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

pub fn get_arc_game_wrapper<P: Player + Clone>(game: GameWrapper<P>) -> ArcGameWrapper<P> {
    Arc::new(RwLock::new(game))
}

#[derive(Clone)]
pub struct GameDB<P: Player + Clone>{
    db: Arc<RwLock<HashMap<String, ArcGameWrapper<P>>>>
}

impl<P: Player + Clone> GameDB<P>{
    pub fn new() -> Self{
        GameDB{db: Arc::new(RwLock::new(HashMap::new()))}
    }

    pub fn get_num_games(&self) -> Result<usize>{
        Ok(self.db.read().unwrap().len())
    }

    pub fn add_new_game(&self, game_id: &str, game: GameWrapper<P>) -> Result<()> {
        eprintln!("Adding game: {}", game_id);
        match self.db.write(){
            Ok(mut db) => {db.insert(String::from(game_id), get_arc_game_wrapper(game));},
            Err(_e) => {bail!("cannot add game now");}
        };
        Ok(())
    }

    pub fn get_game(&self, game_id: &str) -> Result<ArcGameWrapper<P>> {
        match self.db.read(){
            Ok(h_map) => {
                match h_map.get(game_id){
                    Some(arc_game) => {
                        return Ok(arc_game.clone());
                    },
                    None => {
                        bail!(format!("No game found for: {:}", game_id));
                    }
                }
            },
            Err (e) => {
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

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

mod filters{
    use codeword::players::Player;
use crate::GameDB;
    use warp::Filter;
    use super::handlers;

    pub fn app<P: Player + Clone + Send + Sync>(db: GameDB<P>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        return lobby(db.clone()).or(create_player(db.clone()));
    }

    pub fn lobby<P: Player + Clone + Send + Sync>(db: GameDB<P>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("lobby")
            .and(warp::filters::method::post())
            .and(with_db(db))
            .map(handlers::create_lobby)
    }

    pub fn create_player<P: Player + Clone + Send + Sync>(db: GameDB<P>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("lobby" / String / "players")
            .and(warp::path::end())
            .and(warp::filters::method::post())
            .and(warp::body::json())
            .and(with_db(db))
            .map(handlers::create_player)
    }

    fn with_db<P: Player + Clone + Send + Sync>(db: GameDB<P>) -> impl Filter<Extract = (GameDB<P>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}

mod handlers {
    
    use codeword::players::SimplePlayer;
use crate::GameWrapper;
    use codeword::players::Player;
    use crate::GameDB;
    use codeword::game::Game;
    use codeword::game::InitialGame;
    

    pub fn generate_game_id<P: Player + Clone>(db: &GameDB<P>) -> String{
        String::from("game-") + &(match db.get_num_games(){
            Ok(n) => format!("{}", n),
            Err(_) => String::from("-err")
        } as String)
    }

    pub fn create_lobby<P: Player + Clone>(db: GameDB<P>) -> String {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let game: Game<InitialGame, P> = match Game::new(&words){
            Ok(g) => {g},
            Err(e) => {
                eprintln!("Error initializaing game: {:?}", e);
                return String::from("Couldnt init game;")
            },
        };
        let game_id = generate_game_id(&db);
        match db.add_new_game(&game_id, GameWrapper::InitialGame(game)){
            Ok(_) => {return String::from(format!("Game ready!: {}", game_id));},
            Err(e) => {
                println!("Error adding game \"{}\" to DB: {:?}", game_id, e);
                return String::from("Error!");
            },
        };
    }

    pub fn get_player_from_json<P: Player + Clone>(json_data: &str) -> P {
        let player: P = SimplePlayer::new(json_data.as_ref(), 1);
        return player;
    }

    pub fn create_player<P: Player + Clone>(lobby_id: String, json_data: String, db: GameDB<P>) -> String {
        let arc_game = db.get_game(&lobby_id);
        let player = get_player_from_json(json_data.as_ref());
        match arc_game{
            Ok(game) => {
                println!("Creating player for: {}", lobby_id);
                println!("Creating player: {}", json_data);
                match game.write(){
                    Ok(g) => {
                        match *g {
                            GameWrapper::InitialGame(g0) => {
                                g0.add_player_team_one(player);
                            },
                            GameWrapper::InProgressGame(g0) => {
                                g0.add_player_team_one(player);
                            },
                        };
                        return String::from("Ok");
                    },
                    Err(e) => {
                        return String::from("Error adding player");
                    }
                }
            },
            Err(e) => {
                return String::from("Error adding player");
            }
        }

    }
}
