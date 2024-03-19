use std::{collections::HashMap, sync::{Arc, RwLock}};
use lazy_static::lazy_static;
use tokio::sync::mpsc::{Sender, Receiver};

lazy_static! {
static ref PIPECTL: NamedPipe = NamedPipe { map: HashMap::new() };
}

pub struct NamedPipe {
    map: HashMap<String, (Arc<RwLock<Sender<PipeObject>>>, Arc<RwLock<Receiver<PipeObject>>>)>,
}

impl NamedPipe {
    pub fn gen_sender(name: String) -> Arc<RwLock<Sender<PipeObject>>> {
        if PIPECTL.map.contains_key(&name) {}
    }
}

pub enum PipeObject {}

unsafe impl Sync for PipeObject {}
