use futures::Future;
use lavalink::player::*;
use std::cell::RefCell;
use std::rc::Rc;
use super::{Node, NodeConfig};
use websocket::async::Handle;
use ::{Error, EventHandler};

pub struct NodeManager {
    handle: Handle,
    handler: Rc<Box<EventHandler>>,
    pub nodes: Rc<RefCell<Vec<Node>>>,
    pub player_manager: Rc<AudioPlayerManager>,
}

impl NodeManager {
    pub fn new(handle: Handle, handler: Box<EventHandler>) -> Self {
        Self {
            nodes: Rc::new(RefCell::new(Vec::new())),
            player_manager: Rc::new(AudioPlayerManager::default()),
            handle,
            handler: Rc::new(handler),
        }
    }

    pub fn add_node<'a>(&'a mut self, config: &'a NodeConfig)
        -> impl Future<Item = (), Error = Error> + 'a {
        let handle = self.handle.clone();
        let nodes = Rc::clone(&self.nodes);

        Node::connect(
            &self.handle,
            config,
            Rc::clone(&self.player_manager),
            Rc::clone(&self.handler),
        ).map(move |node| {
            nodes.borrow_mut().push(node);
        }).map_err(From::from)
    }

    pub fn close(&mut self) {
        for node in self.nodes.borrow_mut().iter_mut() {
            if let Err(why) = node.close() {
                error!("Failed to close node: {:?}", why);
            }
        }
    }
}

impl Drop for NodeManager {
    fn drop(&mut self) {
        self.close();
    }
}
