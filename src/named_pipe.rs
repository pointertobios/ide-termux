use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    RwLock,
};

use crate::components::editor::Editing;

lazy_static! {
    static ref PIPECTL: Arc<RwLock<NamedPipe>> = Arc::new(RwLock::new(NamedPipe {
        map: HashMap::new()
    }));
}

pub struct NamedPipe {
    map: HashMap<
        String,
        (
            Arc<RwLock<Sender<PipeObject>>>,
            Arc<RwLock<Receiver<PipeObject>>>,
        ),
    >,
}

impl NamedPipe {
    pub fn open_sender(name: String) -> Arc<RwLock<Sender<PipeObject>>> {
        if { PIPECTL.blocking_read().map.contains_key(&name) } {
            Arc::clone(&PIPECTL.blocking_read().map.get(&name).unwrap().0)
        } else {
            let (sender, receiver) = mpsc::channel(4);
            let sender = Arc::new(RwLock::new(sender));
            PIPECTL
                .blocking_write()
                .map
                .insert(name, (Arc::clone(&sender), Arc::new(RwLock::new(receiver))));
            sender
        }
    }

    pub fn open_receiver(name: String) -> Arc<RwLock<Receiver<PipeObject>>> {
        if { PIPECTL.blocking_read().map.contains_key(&name) } {
            Arc::clone(&PIPECTL.blocking_read().map.get(&name).unwrap().1)
        } else {
            let (sender, receiver) = mpsc::channel(4);
            let receiver = Arc::new(RwLock::new(receiver));
            PIPECTL
                .blocking_write()
                .map
                .insert(name, (Arc::new(RwLock::new(sender)), Arc::clone(&receiver)));
            receiver
        }
    }
}

pub enum PipeObject {
    Editing(Arc<RwLock<Editing>>),
}

unsafe impl Sync for PipeObject {}
