use indexmap::map::Entry;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tcprs::{MsgFromPlayerToGame, msg_to_player};
use tokio::sync::mpsc::Receiver;

use crate::player::{Player, Players};

#[derive(Debug, Deserialize)]
enum MsgIn {
    Connect,
}

#[derive(Debug, Serialize)]
enum MsgOut {
    Connected {
        number_of_players: usize,
        already_connected: usize,
    },
    AlreadyConnected,
    UnkownMessage(String),
}

pub async fn wait_for_players_to_join(
    to_game_rx: &mut Receiver<MsgFromPlayerToGame>,
    number_of_players: usize,
    starting_hand_size: usize,
) -> Players {
    let mut players = Players::new();

    while let Some(msg) = to_game_rx.recv().await {
        let player = msg.player;
        match serde_json::from_str::<MsgIn>(&msg.msg) {
            Ok(MsgIn::Connect) => match players.entry(player.uuid) {
                Entry::Occupied(_occupied_entry) => {
                    warn!("Player `{}` already connected!", player.name);
                    msg_to_player(&player, MsgOut::AlreadyConnected).await;
                }
                Entry::Vacant(vacant_entry) => {
                    info!("Player `{}` connected!", player.name);
                    vacant_entry.insert(Player::new(player.clone(), starting_hand_size));
                    msg_to_player(
                        &player,
                        MsgOut::Connected {
                            number_of_players,
                            already_connected: players.len(),
                        },
                    )
                    .await;
                    if number_of_players == players.len() {
                        break;
                    }
                }
            },
            Err(err) => {
                warn!(
                    "Player `{}` sent unkown message: `{}`. The error: `{}`",
                    player.name, msg.msg, err
                );
                msg_to_player(&player, MsgOut::UnkownMessage(err.to_string())).await;
            }
        }
    }
    players
}
