pub mod filters {

    use super::handlers;
    use crate::web::db::InMemGameDB;
    use crate::web::cookies::SESSION_ID;
    use warp::filters::ws::ws;
    use warp::Filter;

    pub fn app(
        db: InMemGameDB,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        return create_lobby_filter(db.clone())
            .or(create_player(db.clone()))
            .or(player_websockets(db.clone()));
    }

    pub fn create_lobby_filter(
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

    pub fn player_websockets(
        db: InMemGameDB,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("lobby" / String / "ws")
            .and(ws())
            .and(with_db(db))
            // .map(|lobby_id: String, ws: warp::ws::Ws, db: InMemGameDB| {
            //     ws.on_upgrade(|websocket|async move {
            //         println!("CONNECTED!");
            //     })
            // })
            .and_then(handlers::handle_ws_conn)
    }

    pub fn get_game(
        db: InMemGameDB
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("lobby" / String / "game_info")
            .and(warp::filters::cookie::cookie("SESSION_ID"))
            .and(with_db(db))
            .and_then(handlers::get_game_info)
    }

    fn with_db(
        db: InMemGameDB,
    ) -> impl Filter<Extract = (InMemGameDB,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}

pub mod handlers {

    use crate::web::tasks::spawn_lobby_ws_listen_task;
    use crate::web::tasks::spawn_lobby_death_timer;
    use crate::players::{SimplePlayer, Player};
    use crate::web::responses::CreatePlayerResp;
    use crate::web::lobby::Lobby;
    use crate::web::db::InMemGameDB;
    use crate::web::responses::{OpStatus};
    use crate::web::lobby::GameWrapper;
    use crate::web::cookies::gen_auth_cookie;
    use anyhow::Result;
    use std::time::Duration;
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
        // create lobby and make sure core things work.
        let mut lobby = Lobby::new(&game_id, &(Vec::new()), game);
        let lobby_ws_rcvr = lobby.get_ws_receiver();
        let lobby_ws_rcvr = lobby_ws_rcvr.map_err(|e| warp::reject::custom(e))?;
        // all good, add to DB.
        let add_res = db.add_new_lobby(lobby).await;
        match add_res {
            Ok(_) => {
                // kick start the lobby websocket listening loop.
                let lobby_expire = Duration::from_secs(120);
                spawn_lobby_ws_listen_task(db.clone(), &game_id, lobby_ws_rcvr);
                spawn_lobby_death_timer(db.clone(), &game_id, lobby_expire);
                // return response
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
        mut player: SimplePlayer,
        db: InMemGameDB,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let lobby_res = db.get_lobby(&lobby_id).await;
        if lobby_res.is_err() {
            return Err(warp::reject::not_found());
        }

        let lobby = lobby_res.unwrap();
        let lobby_writer = lobby.read().await;
        let num_players = lobby_writer.get_num_players().await;
        player.set_id(num_players as u32);
        let pid = player.get_id().clone();
        let auth_challenge = lobby_writer.add_player(player.into()).await;
        // TODO: add cookie.
        let reply = warp::reply::json(&CreatePlayerResp{
            status: OpStatus::Ok,
            challenge: auth_challenge,
        });
        // TODO: store session IDs :P
        let sess_id = format!("{}_{}", pid, Uuid::new_v4().to_string());
        return Ok(
            warp::reply::with_header(reply, "set_cookie", gen_auth_cookie(&sess_id))
        );
    }

    pub async fn handle_ws_conn(
        lobby_id: String,
        ws: warp::ws::Ws,
        db: InMemGameDB,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let lobby_res = db.get_lobby(&lobby_id).await;
        if lobby_res.is_err() {
            return Err(warp::reject::not_found());
        }

        let lobby = lobby_res.unwrap();
        Ok(ws.on_upgrade(|websocket| async move {
            eprintln!("Upgrading websocket for lobby: {}", lobby_id);
            let mut lobby_writer = lobby.write().await;
            (*lobby_writer).handle_incoming_ws(websocket).await;
        }))
    }

    pub async fn get_game_info(lobby_id: String, sess_id: String, db: InMemGameDB) -> Result<String, warp::Rejection> {
        Ok("Cool".to_string())
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

#[cfg(test)]
mod tests {

    use super::filters::*;
    use crate::web::db::InMemGameDB;

    #[tokio::test]
    async fn test_ws_conn() {
        let mut db = InMemGameDB::new();
        let new_lobby_route = create_lobby_filter(db.clone());
        let res = warp::test::request()
            .path("/lobby")
            .method("POST")
            .reply(&new_lobby_route)
            .await;
        let lobby_id = String::from_utf8(res.body().to_vec()).expect("Response encoding error");
        // assert!(db.get_num_get_num_lobbies().await, Ok(1));
        let route = player_websockets(db.clone());
        let ws_path = format!("/lobby/{}/ws", &lobby_id);
        println!("WS: {}", ws_path);
        let _client = warp::test::ws()
            .path(&ws_path)
            .handshake(route)
            .await
            .expect("handshake");
        let lobby = db
            .get_lobby(&lobby_id)
            .await
            .expect("Should have found lobby");
        {
            let lobby_rdr = lobby.read().await;
            assert_eq!(1, (*lobby_rdr).get_num_unidentified_ws().await);
        }
        db.drop_lobby(&lobby_id).await;
    }
}
