use std::collections::HashMap;

use tcprs::TcpRsPlayer;
use uuid::Uuid;

use crate::hand::Hand;

#[derive(Debug)]
pub struct Player {
    pub tcprs_player: TcpRsPlayer,
    pub hand: Hand,
}

impl Player {
    pub fn new(tcprs_player: TcpRsPlayer) -> Self {
        Self {
            tcprs_player,
            hand: Hand::empty(),
        }
    }
}

pub type Players = HashMap<Uuid, Player>;
