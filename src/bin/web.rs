use warp::Filter;

#[tokio::main]
async fn main() {
    println!("Web App!");
    let api = filters::app();
    let routes = api.with(warp::log("codeword"));
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

mod filters{
    use warp::Filter;
    use super::handlers;

    pub fn app() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        return lobby().or(create_player());
    }

    pub fn lobby() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("lobby")
            .and(warp::filters::method::post())
            .map(handlers::create_lobby)
    }

    pub fn create_player() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("lobby" / String / "players")
            .and(warp::path::end())
            .and(warp::filters::method::post())
            .and(warp::body::json())
            .map(handlers::create_player)
    }
}

mod handlers {
    use codeword::game::Game;
    use codeword::game::InitialGame;
    use codeword::players::SimplePlayer;


    pub fn create_lobby() -> String {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let game: Game<InitialGame, SimplePlayer> = match Game::new(&words){
            Ok(g) => {g},
            Err(e) => {
                eprintln!("Error initializaing game: {:?}", e);
                return String::from("Couldnt init game;")
            },
        };
        println!("{:?}", game);
        return String::from("Game ready!");
    }

    pub fn create_player(lobby_id: String, json_data: String) -> String {
        println!("Creating player for: {}", lobby_id);
        println!("Creating player: {}", json_data);
        return String::from("Player Created!");
    }
}
