mod die;
mod game;
mod hand;
mod player;
mod wait_for_players_to_join;

use tcprs::{TcpRsConfig, run_tcp_server};

use crate::{
    game::game_loop,
    wait_for_players_to_join::{notify_players_game_started, wait_for_players_to_join},
};

// TODO: Read from config file or argument
const IP_PORT: &str = "127.0.0.1:5942";
const BUFFER_SIZE: usize = 1024;
const NUMBER_OF_PLAYERS: usize = 2;

#[tokio::main]
async fn main() {
    env_logger::init();
    let tcprs_config = TcpRsConfig::new(IP_PORT, BUFFER_SIZE, BUFFER_SIZE);
    let mut to_game_rx = run_tcp_server(tcprs_config)
        .await
        .expect("Unable to start Server");
    let players = wait_for_players_to_join(&mut to_game_rx, NUMBER_OF_PLAYERS).await;
    notify_players_game_started(&players).await;
    game_loop(&mut to_game_rx, players).await;
}
