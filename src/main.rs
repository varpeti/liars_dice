mod die;
mod game;
mod guess;
mod hand;
mod player;
mod wait_for_players_to_join;

use tcprs::{TcpRsConfig, run_tcp_server};

use crate::game::{Game, GameConfig};

// TODO: Read from config file or argument
const IP_PORT: &str = "127.0.0.1:5942";
const BUFFER_SIZE: usize = 1024;
const NUMBER_OF_PLAYERS: usize = 2;
const TURN_DURATION_MS: u64 = 20000;
const STARTING_HAND_SIZE: usize = 5;

#[tokio::main]
async fn main() {
    env_logger::init();

    let tcprs_config = TcpRsConfig::new(IP_PORT, BUFFER_SIZE, BUFFER_SIZE);
    let to_game_rx = run_tcp_server(tcprs_config)
        .await
        .expect("Unable to start Server");

    let game_config = GameConfig {
        turn_duration_ms: TURN_DURATION_MS,
        number_of_players: NUMBER_OF_PLAYERS,
        starting_hand_size: STARTING_HAND_SIZE,
    };
    let mut game = Game::new(game_config, to_game_rx);
    game.run().await;
}
