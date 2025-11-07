use std::{collections::HashMap, time::Duration};

use anyhow::{Result, anyhow};
use log::{info, warn};
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use tcprs::{MsgFromPlayerToGame, msg_to_player};
use tokio::{sync::mpsc::Receiver, time::timeout};
use uuid::Uuid;

use crate::{
    die::Die,
    guess::Guess,
    player::{Player, Players},
    wait_for_players_to_join::wait_for_players_to_join,
};

#[derive(Debug, Deserialize)]
enum MsgIn {
    Connect,
    Disconnect,
}

#[derive(Debug, Deserialize)]
enum Action {
    IThinkThereAre { number: u8, face: u8 },
    Liar,
    Exactly,
}

#[derive(Debug, Serialize, Clone)]
enum MsgOut {
    // Notifications
    GameStarted,
    GameEnded {
        winner: String,
    },
    // Admin
    Reconnected,
    GameAlreadyStarted,
    UnkownMessage(String),
    NotYourTurn,
    // Responses
    Turn {
        turn: usize,
        number: u8,
        face: u8,
        next_player: String,
    },
    Round {
        result: String,
        revealed_hands: HashMap<String, Vec<u8>>,
    },
    YouRolled {
        hand: Vec<u8>,
    },
    YourTurn, // TODO:
}

pub struct GameConfig {
    pub turn_duration_ms: u64,
    pub number_of_players: usize,
    pub starting_hand_size: usize,
}

pub struct Game {
    config: GameConfig,
    to_game_rx: Receiver<MsgFromPlayerToGame>,
    players: Players,
    turn: usize,
    current_guess: Guess,
    number_of_remaining_players: usize,
    rng: ThreadRng,
}

impl Game {
    pub fn new(config: GameConfig, to_game_rx: Receiver<MsgFromPlayerToGame>) -> Self {
        let number_of_remaining_players = config.number_of_players;
        Self {
            config,
            to_game_rx,
            players: Players::new(),
            turn: 0,
            current_guess: Guess::start(),
            number_of_remaining_players,
            rng: rand::rng(),
        }
    }

    pub async fn run(&mut self) {
        self.players = wait_for_players_to_join(
            &mut self.to_game_rx,
            self.config.number_of_players,
            self.config.starting_hand_size,
        )
        .await;
        notify_players(&self.players, MsgOut::GameStarted).await;
        self.roll().await;
        self.turn(Guess::start()).await;
        self.game_loop().await;
        let winner = &self.players[0];
        notify_players(
            &self.players,
            MsgOut::GameEnded {
                winner: winner.tcprs_player.name.clone(),
            },
        )
        .await;
    }

    async fn game_loop(&mut self) {
        while self.number_of_remaining_players > 1 {
            let mut action_done = false;
            while let Ok(Some(msg)) = timeout(
                Duration::from_millis(self.config.turn_duration_ms),
                self.to_game_rx.recv(),
            )
            .await
            {
                let player = match self.players.get(&msg.player.uuid) {
                    Some(player) => player.clone(),
                    None => {
                        warn!(
                            "New Player `{}` tried to connect, but game is alreay started!",
                            msg.player.name
                        );
                        msg_to_player(&msg.player, MsgOut::GameAlreadyStarted).await;
                        continue;
                    }
                };

                if player.tcprs_player.uuid != self.get_current_player().tcprs_player.uuid {
                    msg_to_player(&player.tcprs_player, MsgOut::NotYourTurn).await;
                    continue;
                }

                if let Ok(action) = serde_json::from_str::<MsgIn>(&msg.msg) {
                    match action {
                        MsgIn::Connect => {
                            info!(
                                "Player `{}` -> `{}` reconnected!",
                                player.tcprs_player.name, msg.player.name
                            );
                            // Update player data, we are sure at this point that this player is connected/exists
                            self.get_player_mut(&player.tcprs_player.uuid)
                                .unwrap()
                                .tcprs_player = msg.player;
                            msg_to_player(&player.tcprs_player, MsgOut::Reconnected).await;
                        }
                        MsgIn::Disconnect => {
                            warn!("Player `{}` disconnected!", player.tcprs_player.name)
                        }
                    }
                }

                match serde_json::from_str::<Action>(&msg.msg) {
                    Ok(action) => match action {
                        Action::IThinkThereAre { number, face } => {
                            self.action_i_think_there_are(Guess::new(number, face))
                                .await;
                            action_done = true;
                            break;
                        }
                        Action::Liar => {
                            self.action_liar().await;
                            action_done = true;
                            break;
                        }
                        Action::Exactly => {
                            self.action_exactly().await;
                            action_done = true;
                            break;
                        }
                    },
                    Err(err) => {
                        warn!(
                            "Player `{}` sent unkown message: `{}`. The error: `{}`",
                            player.tcprs_player.name, msg.msg, err
                        );
                        msg_to_player(&player.tcprs_player, MsgOut::UnkownMessage(err.to_string()))
                            .await;
                    }
                }
            }
            if !action_done {
                self.action_idle().await;
            }
        }
    }

    async fn action_i_think_there_are(&mut self, guess: Guess) {
        let invalid_bid = "they placed an InvalidBid:";
        if guess.face < 1 || guess.face > 6 {
            self.round(
                true,
                format!("{invalid_bid} Face must be between 1 and 6"),
                true,
            )
            .await;
            return;
        }
        if self.current_guess.face != 1 {
            // Normal bid
            let mut number = self.current_guess.number;
            if guess.face == 1 {
                number = number.div_ceil(2);
            }
            if !(number < guess.number || self.current_guess.face < guess.face) {
                self.round(
                    true,
                    format!(
                        "{} Number `{}<{}` or Face `{}<{}` must be increased!",
                        invalid_bid, number, guess.number, self.current_guess.face, guess.face
                    ),
                    true,
                )
                .await;
                return;
            }
        } else {
            // Joker bid
            if guess.face == 1 && self.current_guess.number >= guess.face {
                self.round(
                    true,
                    format!("{invalid_bid} Number must be increased on Joker bid!"),
                    true,
                )
                .await;
                return;
            } else if (self.current_guess.number * 2) >= guess.face {
                self.round(
                    true,
                    format!("{invalid_bid} Number must be Double +1 or higher on Joker bid when changing Face!"),
                    true,
                )
                .await;
                return;
            }
        }
        // Bid is valid
        self.turn(guess).await;
    }

    async fn turn(&mut self, guess: Guess) {
        self.turn += 1;
        self.current_guess = Guess::new(guess.number, guess.face);
        let next_player = self.get_current_player().tcprs_player.name.clone();
        notify_players(
            &self.players,
            MsgOut::Turn {
                turn: self.turn,
                number: guess.number,
                face: guess.face,
                next_player,
            },
        )
        .await;
    }

    async fn action_liar(&mut self) {
        let mut sum = 0;
        for player in self.players.values() {
            for die in player.hand.dice.iter() {
                if die.face == 1 || die.face == self.current_guess.face {
                    sum += 1;
                }
            }
        }

        let (is_current_player, result) = match sum.cmp(&self.current_guess.number) {
            std::cmp::Ordering::Less => (
                false,
                format!(
                    "there is less `{}` face `{}` than guessed `{}`",
                    sum, self.current_guess.face, self.current_guess.number
                ),
            ),
            std::cmp::Ordering::Equal => (
                true,
                format!(
                    "there are eaxtly `{}` face `{}` ",
                    sum, self.current_guess.face
                ),
            ),
            std::cmp::Ordering::Greater => (
                true,
                format!(
                    "there are more `{}` face `{}` than guessed `{}`",
                    sum, self.current_guess.face, self.current_guess.number
                ),
            ),
        };

        self.round(is_current_player, result, true).await;
    }

    async fn action_exactly(&mut self) {
        let mut sum = 0;
        for player in self.players.values() {
            for die in player.hand.dice.iter() {
                if die.face == 1 || die.face == self.current_guess.face {
                    sum += 1;
                }
            }
        }

        let (lost, result) = match sum.cmp(&self.current_guess.number) {
            std::cmp::Ordering::Less => (
                true,
                format!(
                    "there is less `{}` face `{}` than guessed `{}`",
                    sum, self.current_guess.face, self.current_guess.number
                ),
            ),
            std::cmp::Ordering::Equal => (
                false,
                format!(
                    "there are eaxtly `{}` face `{}` ",
                    sum, self.current_guess.face
                ),
            ),
            std::cmp::Ordering::Greater => (
                true,
                format!(
                    "there are more `{}` face `{}` than guessed `{}`",
                    sum, self.current_guess.face, self.current_guess.number
                ),
            ),
        };
        self.round(true, result, lost).await;
    }

    async fn action_idle(&mut self) {
        let reason = "they did not take action in time!".to_string();
        self.round(true, reason, true).await;
    }

    async fn round(&mut self, is_current_player: bool, mut result: String, lost: bool) {
        let mut revealed_hands = HashMap::new();
        for player in self.players.values() {
            revealed_hands.insert(
                player.tcprs_player.name.clone(),
                player.hand.dice.iter().map(|d| d.face).collect(),
            );
        }

        let player = if is_current_player {
            self.get_current_player_mut()
        } else {
            self.get_previous_player_mut()
        };

        if lost {
            result = format!(
                "Player `{}` lost one die because {}",
                player.tcprs_player.name, result
            );
            player.hand.dice.pop();

            if player.hand.dice.is_empty() {
                let uuid = player.tcprs_player.uuid;
                let player = self
                    .players
                    .swap_remove(&uuid)
                    .expect("At least one Player");
                self.players.insert(uuid, player);
                self.number_of_remaining_players -= 1;
            }
        } else {
            result = format!(
                "Player `{}` gained one extra die because {}",
                player.tcprs_player.name, result
            );
            player.hand.dice.push(Die { face: 1 });
        }

        notify_players(
            &self.players,
            MsgOut::Round {
                result,
                revealed_hands,
            },
        )
        .await;

        self.roll().await;
        self.turn(Guess::start()).await
    }

    async fn roll(&mut self) {
        for player in self.players.values_mut() {
            player.hand.roll(&mut self.rng);
            msg_to_player(
                &player.tcprs_player,
                MsgOut::YouRolled {
                    hand: player.hand.dice.iter().map(|d| d.face).collect(),
                },
            )
            .await
        }
    }

    fn get_current_player(&self) -> &Player {
        match self
            .players
            .get_index(self.turn % self.config.number_of_players)
        {
            Some((_uuid, player)) => player,
            None => {
                panic!(
                    "Current Player not found for turn `{} ({})`!",
                    self.turn,
                    self.turn % self.config.number_of_players
                );
            }
        }
    }

    fn get_current_player_mut(&mut self) -> &mut Player {
        match self
            .players
            .get_index_mut(self.turn % self.config.number_of_players)
        {
            Some((_uuid, player)) => player,
            None => {
                panic!(
                    "Current Player not found for turn `{} ({})`!",
                    self.turn,
                    self.turn % self.config.number_of_players
                );
            }
        }
    }

    fn get_previous_player_mut(&mut self) -> &mut Player {
        let turn = self.turn.saturating_sub(1);
        match self
            .players
            .get_index_mut(turn % self.config.number_of_players)
        {
            Some((_uuid, player)) => player,
            None => {
                panic!(
                    "Pervious Player not found for turn `{} ({})`!",
                    self.turn,
                    turn % self.config.number_of_players
                );
            }
        }
    }

    fn get_player_mut(&mut self, player_uuid: &Uuid) -> Result<&mut Player> {
        match self.players.get_mut(player_uuid) {
            Some(player) => Ok(player),
            None => Err(anyhow!("Player not found by uuid `{}`!", player_uuid)),
        }
    }
}

async fn notify_players(players: &Players, msg: MsgOut) {
    for (_, player) in players {
        msg_to_player(&player.tcprs_player, msg.clone()).await;
    }
}
