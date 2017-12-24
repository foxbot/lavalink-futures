use futures::Future;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::i32;
use super::{Node, NodeConfig};
use websocket::async::Handle;
use ::player::{AudioPlayerManager, AudioPlayer};
use ::{Error, EventHandler};

pub struct NodeManager {
    handle: Handle,
    handler: Rc<Box<EventHandler>>,
    pub nodes: Rc<RefCell<HashMap<String, Node>>>,
    pub player_manager: Rc<AudioPlayerManager>,
}

impl NodeManager {
    pub fn new(handle: Handle, handler: Box<EventHandler>) -> Self {
        Self {
            nodes: Rc::new(RefCell::new(HashMap::new())),
            player_manager: Rc::new(AudioPlayerManager::default()),
            handle,
            handler: Rc::new(handler),
        }
    }

    /// Adds a new node to be managed.
    ///
    /// This will add the node to [`nodes`] once the connection successfully
    /// resolves.
    ///
    /// Resolves to an error if there was a problem connecting to the node.
    ///
    /// [`nodes`]: #structfield.nodes
    pub fn add<'a>(&'a mut self, config: &'a NodeConfig)
        -> impl Future<Item = (), Error = Error> + 'a {
        let handle = self.handle.clone();
        let nodes = Rc::clone(&self.nodes);

        Node::connect(
            &self.handle,
            config,
            Rc::clone(&self.player_manager),
            Rc::clone(&self.handler),
        ).map(move |node| {
            nodes.borrow_mut().insert(config.websocket_host.clone(), node);
        }).map_err(From::from)
    }

    /// Determines the best node, if any.
    ///
    /// This does not return the node, but does return the websocket host (keyed
    /// in [`nodes`]).
    ///
    /// [`nodes`]: #structfield.nodes
    pub fn best_node(&self) -> Option<&str> {
        let mut record = i32::MAX;
        let mut best = None;

        let nodes = (Rc::try_unwrap(self.nodes).ok()?).try_borrow().ok()?;

        nodes.iter().for_each(|(name, node)| {
            let total = node.penalty().unwrap_or(0);

            if total < record {
                best = Some(&name[..]);
                record = total;
            }
        });

        best
    }

    /// Closes a node by websocket host.
    ///
    /// Returns whether closing the node was successful. This can fail if the
    /// node is not recognized by host.
    pub fn close(&mut self, websocket_host: &str) -> bool {
        match self.nodes.borrow_mut().get(websocket_host) {
            Some(host) => {
                host.close();

                true
            },
            None => false,
        }
    }

    /// Closes all of the nodes owned by the manager.
    ///
    /// This is also automatically called when the instance is dropped.
    // **NOTE**: This can _NOT_ call `close`, as `nodes` is already mutably
    // borrowed here.
    pub fn close_all(&mut self) {
        self.nodes.borrow_mut().values_mut().for_each(|node| {
            if let Err(why) = node.close() {
                error!("Failed to close node: {:?}", why);
            }
        });
    }

    pub fn create_player(
        &mut self,
        guild_id: u64,
        node_websocket_host: Option<&str>,
    ) -> Option<&mut AudioPlayer> {
        let manager = Rc::get_mut(&mut self.player_manager)?;

        let node = match node_websocket_host {
            Some(host) => self.get_node(host)?,
            None => self.get_node(self.best_node()?)?,
        };

        manager.create(guild_id, node.user_to_node.clone()).ok()
    }

    pub fn get_node(&self, websocket_host: &str) -> Option<&Node> {
        self.nodes.try_borrow().ok()?.get(websocket_host)
    }
}

impl Drop for NodeManager {
    /// Drops the manager, closing all nodes if possible.
    fn drop(&mut self) {
        self.close_all();
    }
}
