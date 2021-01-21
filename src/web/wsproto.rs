/// Websocket protocol
use warp::ws::Message;
use serde::{Serialize, Deserialize};

pub use crate::web::auth::{AuthResponse, AuthChallenge};

/// Wrapper for all Websocket messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WSMessage{
    /// Auth response
    AuthResponse(AuthResponse),
    /// catch all invalid messages
    InvalidMessage,
}

impl From<Message> for WSMessage{
    fn from(msg: Message) -> Self { 
        match msg.to_str(){
            Ok(msg_str) => {
                let tmp = serde_json::from_str(msg_str);
                match tmp{
                    Ok(wm) => wm,
                    Err(_) => {Self::InvalidMessage}
                }
            },
            Err(_) => {
                WSMessage::InvalidMessage
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_ser_deser() {
        let test_msg = AuthResponse{pid: 1, response: "abc123".to_string()};
        let msg = WSMessage::AuthResponse(test_msg.clone());
        assert_eq!(
            "{\"AuthResponse\":{\"pid\":1,\"response\":\"abc123\"}}",
            serde_json::to_string(&msg).unwrap().as_str()
        );
        let msg_str = format!("{{\"AuthResponse\": {}}}", serde_json::to_string(&test_msg).unwrap());
        let msg = serde_json::from_str(&msg_str).unwrap();
        match msg{
            WSMessage::AuthResponse(auth_resp) => {
                assert_eq!(test_msg, auth_resp);
            },
            m => {
                let err = format!("Invalid msg: {:?}", m);
                assert!(false, err);
            }
        }
    }
}
