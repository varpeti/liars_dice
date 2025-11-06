use indexmap::IndexMap;
use tcprs::TcpRsPlayer;
use uuid::Uuid;

use crate::hand::Hand;

#[derive(Debug, Clone)]
pub struct Player {
    pub tcprs_player: TcpRsPlayer,
    pub hand: Hand,
}

impl Player {
    pub fn new(tcprs_player: TcpRsPlayer, starting_hand_size: usize) -> Self {
        Self {
            tcprs_player,
            hand: Hand::new(starting_hand_size),
        }
    }
}

pub type Players = IndexMap<Uuid, Player>;
