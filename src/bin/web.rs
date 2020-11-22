
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

#[derive(Clone)]
pub struct GameDB<P: Player + Clone>{
    db: Arc<RwLock<HashMap<String, GameWrapper<P>>>>
}

impl<P: Player + Clone> GameDB<P>{
    pub fn new() -> Self{
        GameDB{db: Arc::new(RwLock::new(HashMap::new()))}
    }

    pub fn get_num_games(&self) -> Result<usize>{
        Ok(self.db.read().unwrap().len())
    }

    pub fn add_new_game(&self, game_id: &str, game: GameWrapper<P>) -> Result<()> {
        match self.db.write(){
            Ok(mut db) => {db.insert(String::from(game_id), game);},
            Err(_e) => {bail!("cannot add game now");}
        };
        Ok(())
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
    
    use crate::GameWrapper;
    use codeword::players::Player;
    use crate::GameDB;
    use codeword::game::Game;
    use codeword::game::InitialGame;
    use codeword::players::SimplePlayer;

    pub fn generate_game_id<P: Player + Clone>(db: GameDB<P>) -> String{
        String::from("game-") + &(match db.get_num_games(){
            Ok(n) => format!("{}", n),
            Err(_) => String::from("-err")
        } as String)
    }

    pub fn create_lobby<P: Player + Clone>(db: GameDB<P>) -> String {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let game: Game<InitialGame, SimplePlayer> = match Game::new(&words){
            Ok(g) => {g},
            Err(e) => {
                eprintln!("Error initializaing game: {:?}", e);
                return String::from("Couldnt init game;")
            },
        };
        let game_id = generate_game_id(db);
        db.add_new_game(&game_id, GameWrapper::InitialGame(game));
        // let game = db.get_mut_game()
        // println!("{:?}", game);
        return String::from("Game ready!");
    }

    pub fn create_player<P: Player + Clone>(lobby_id: String, json_data: String, db: GameDB<P>) -> String {
        println!("Creating player for: {}", lobby_id);
        println!("Creating player: {}", json_data);
        return String::from("Player Created!");
    }
}
