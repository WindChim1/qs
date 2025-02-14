use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use libp2p::{Multiaddr, PeerId};

#[derive(Default)]
pub struct SharedState {
    pub active_passwords: HashMap<String, (PathBuf, Vec<Multiaddr>)>, // 口令 -> 文件路径/text内容
    pub discovered_perrs: HashSet<PeerId>,
    // pub pending_requests: HashMap<PeerId, ResponseChannel<FileResponse>>,
}
