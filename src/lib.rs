use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use behaviour::FileResponse;
use libp2p::{request_response::ResponseChannel, PeerId};

pub mod behaviour;

#[derive(Default)]
pub struct SharedState {
    pub active_passwords: HashMap<String, PathBuf>, // 口令 -> 文件路径/text内容
    pub discovered_perrs: HashSet<PeerId>,
    pub pending_requests: HashMap<PeerId, ResponseChannel<FileResponse>>,
}
