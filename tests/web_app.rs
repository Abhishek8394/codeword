// Larger integration type tests for web app.


use std::time::Duration;
use codeword::web::wsproto::WSMessage;
use codeword::web::wsproto::AuthResponse;

use codeword::web::responses::CreatePlayerResp;
use codeword::web::db::InMemGameDB;
use codeword::web::app::filters;

fn hyper_bytes_to_string(b: &warp::hyper::body::Bytes) -> Result<String, String> {
    Ok(String::from_utf8(b.to_vec()).unwrap())
}

#[tokio::test]
async fn test_player_ws_conn() {
    let db = InMemGameDB::new();
    let web_app = filters::app(db.clone());

    // build lobby
    let req = warp::test::request()
        .path("/lobby")
        .method("POST")
        .reply(&web_app)
        .await;
    assert_eq!(req.status(), 200);
    let lobby_id: String = hyper_bytes_to_string(req.body()).unwrap();
    // setup urls
    let create_player_url = format!("/lobby/{}/players", lobby_id);
    println!("{}", create_player_url);
    let ws_url = format!("/lobby/{}/ws", lobby_id);
    // create players & ws
    let mut players: Vec<CreatePlayerResp> = vec![];
    let mut ws = vec![];
    let num_test_players = 3;
    assert!(num_test_players >= 2);
    for i in 0..num_test_players{
        let player_name = format!("player-{}", i);
        let player_req_body = format!("{{\"name\": \"{}\"}}", player_name);
        // connect player
        let p1 = warp::test::request()
            .path(&create_player_url)
            .method("POST")
            .body(&player_req_body)
            .reply(&web_app)
            .await;
        let p1 = serde_json::from_str(
                &hyper_bytes_to_string(p1.body())
                .expect("cannot parse playe resp")
            ).expect("cannot conv player resp to json");
        players.push(p1);

        // connect ws
        let w = warp::test::ws()
            .path(&ws_url)
            .handshake(web_app.clone())
            .await
            .expect("handshake");
        ws.push(w);
    }
    // pre auth connection checks.
    let lobby = db.get_lobby(&lobby_id).await.expect("lobby not found in db");
    {
        let lobby_rdr = lobby.read().await;
        let n_players = (*lobby_rdr).get_num_players().await;
        assert_eq!(num_test_players, n_players);
        let n_unreg_ws = (*lobby_rdr).get_num_unidentified_ws().await;
        assert_eq!(num_test_players, n_unreg_ws);
    }
    // num players to authenticate
    let num_holdout = 1;
    assert!(num_holdout < num_test_players);
    // auth all except last one.
    for i in 0..(num_test_players - num_holdout){
        let player = &players[i];
        let challenge = &player.challenge;
        // since an echo challenge, send it back as is.
        let challenge_ans = &challenge.challenge;
        // send auth.
        let web_sock = &mut ws[i];
        let ws_msg = serde_json::to_string(&WSMessage::AuthResponse(AuthResponse{
            pid: challenge.pid,
            response: challenge_ans.clone(),
        })).unwrap();
        web_sock.send_text(ws_msg).await;
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let lobby_rdr = lobby.read().await;
            let n_unreg_ws = (*lobby_rdr).get_num_unidentified_ws().await;
            println!("authenticate req: {}, unreg: {}", i, n_unreg_ws);
            assert_eq!(num_test_players - n_unreg_ws, i + 1);
        }
    }

    drop(ws);
    drop(lobby);
    tokio::time::sleep(Duration::from_millis(10)).await;
    let n_lobbies = db.get_num_lobbies().await.unwrap();
    assert_eq!(0, n_lobbies);
}
