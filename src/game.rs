use log::{info, warn};
use serde::{Deserialize, Serialize};
use tcprs::{MsgFromPlayerToGame, msg_to_player};
use tokio::sync::mpsc::Receiver;

use crate::player::Players;

#[derive(Debug, Deserialize)]
enum MsgIn {
    Connect,
    Disconnect,
}

#[derive(Debug, Serialize)]
enum MsgOut {
    Reconnected,
    GameAlreadyStarted,
    UnkownMessage(String),
}

pub async fn game_loop(to_game_rx: &mut Receiver<MsgFromPlayerToGame>, mut players: Players) {
    while let Some(msg) = to_game_rx.recv().await {
        let player = match players.get_mut(&msg.player.uuid) {
            Some(player) => player,
            None => {
                warn!(
                    "New Player `{}` tried to connect, but game is alreay started!",
                    msg.player.name
                );
                msg_to_player(&msg.player, MsgOut::GameAlreadyStarted).await;
                continue;
            }
        };

        match serde_json::from_str::<MsgIn>(&msg.msg) {
            Ok(action) => match action {
                MsgIn::Connect => {
                    info!(
                        "Player `{}` -> `{}` reconnected!",
                        player.tcprs_player.name, msg.player.name
                    );
                    player.tcprs_player = msg.player;
                    msg_to_player(&player.tcprs_player, MsgOut::Reconnected).await;
                }
                MsgIn::Disconnect => {
                    warn!("Player `{}` disconnected!", player.tcprs_player.name)
                }
            },
            Err(err) => {
                warn!(
                    "Player `{}` sent unkown message: `{}`. The error: `{}`",
                    player.tcprs_player.name, msg.msg, err
                );
                msg_to_player(&player.tcprs_player, MsgOut::UnkownMessage(err.to_string())).await;
            }
        }
    }
}
