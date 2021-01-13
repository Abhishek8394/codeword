use crate::web::db::InMemGameDB;
use crate::web::ws::PlayerWebSocketMsg;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;


pub fn spawn_lobby_ws_listen_task(
    db: InMemGameDB,
    game_id: &str,
    mut lobby_ws_rcvr: Receiver<PlayerWebSocketMsg>,
) {
    let game_id: String = game_id.to_string();
    tokio::task::spawn(async move {
        eprintln!("[{:?}] Starting websocket loop", game_id);
        loop {
            if let Ok(lobby) = db.get_lobby(&game_id).await {
                let pws_msg = lobby_ws_rcvr.recv().await;
                match pws_msg {
                    Some(pws_msg) => {
                        if let (uniq_id, Ok(msg)) = pws_msg {
                            let lobby_rdr = lobby.read().await;
                            println!("[{}] Got ({}): {:?}", (*lobby_rdr).get_id(), uniq_id, msg);
                            // TODO:
                            // - match uniq id
                            // - handle auth msg
                            // - handle game msg
                            // for early quit, poll for num players connected?
                        }
                    }
                    // everyone has disconnected, drop out and delete lobby maybe?
                    // If don't want to drop, then remove break.
                    None => break,
                }
            }
        }
        eprintln!("[{:?}] Ended websocket loop", game_id);
    });
}

pub fn spawn_lobby_death_timer(mut db: InMemGameDB, game_id: &str, duration: Duration) {
    let game_id: String = game_id.to_string();
    tokio::task::spawn(async move {
        eprintln!("[{:?}] Starting death timer: {:?}", game_id, duration);
        tokio::time::delay_for(duration).await; // called sleep in 1.0+
        db.drop_lobby(&game_id).await;
    });
}
