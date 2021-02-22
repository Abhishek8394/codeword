use crate::web::db::InMemGameDB;
use crate::web::ws::PlayerWebSocketMsg;
use crate::web::wsproto::WSMessage;
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
                            {
                                let lobby_rdr = lobby.read().await;
                                println!(
                                    "[{}] Got ({}): {:?}",
                                    (*lobby_rdr).get_id(),
                                    uniq_id,
                                    msg
                                );
                            }
                            if msg.is_close() {
                                let lobby_rdr = lobby.read().await;
                                (*lobby_rdr).close_ws(&uniq_id).await;
                                break;
                            }
                            let ws_msg: WSMessage = msg.into();
                            match ws_msg {
                                WSMessage::AuthResponse(auth_resp) => {
                                    let lobby_rdr = lobby.read().await;
                                    (*lobby_rdr).handle_auth_resp(&uniq_id, auth_resp).await;
                                    // tokio::task::spawn();
                                }
                                WSMessage::InvalidMessage => {
                                    // do nothing. Log maybe?
                                }
                                WSMessage::InvalidMove { .. } => {}
                                WSMessage::AuthOk => {}
                                WSMessage::AuthReject => {}
                                WSMessage::UpdateState(_) => {}
                                WSMessage::TeamWinMessage { .. } => {}
                                WSMessage::TileSelect(tile_num) => {
                                    let mut lobby_writer = lobby.write().await;
                                    (*lobby_writer)
                                        .handle_tile_select_msg(&uniq_id, tile_num)
                                        .await;
                                }
                            }
                            // TODO:
                            // - handle game msg
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
        tokio::time::sleep(duration).await;
        db.drop_lobby(&game_id).await;
    });
}
