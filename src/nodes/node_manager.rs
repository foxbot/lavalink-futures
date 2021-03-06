use futures::Future;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::i32;
use super::{Node, NodeConfig};
use websocket::async::Handle;
use ::player::AudioPlayerManager;
use ::{Error, EventHandler};

/// A struct responsible for connecting to Lavalink nodes and providing
/// shortcuts for audio player usage.
pub struct NodeManager {
    handle: Handle,
    handler: Rc<RefCell<Box<EventHandler>>>,
    /// HashMap of nodes, keyed by the websocket host.
    pub nodes: HashMap<String, Node>,
    /// The player manager holding all of the audio players for nodes managed
    /// under the instance of a `NodeManager`.
    pub player_manager: Rc<RefCell<AudioPlayerManager>>,
}

impl NodeManager {
    /// Creates a new NodeManager.
    ///
    /// Requires a handle to the tokio Core in use and an instance of a type
    /// implementing the [`EventHandler`] trait.
    ///
    /// [`EventHandler`]: ../trait.EventHandler.html
    pub fn new(handle: Handle, handler: RefCell<Box<EventHandler>>) -> Self {
        Self {
            nodes: HashMap::new(),
            player_manager: Rc::new(RefCell::new(AudioPlayerManager::default())),
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
    pub fn add_node(mut self, config: NodeConfig)
        -> Box<Future<Item = Self, Error = Error>> {
        let ws_host = config.websocket_host.clone();

        let done = Node::connect(
            self.handle.clone(),
            config,
            Rc::clone(&self.player_manager),
            Rc::clone(&self.handler),
        ).map(move |node| {
            self.nodes.insert(ws_host, node);

            self
        }).map_err(|why| {
            trace!("Err adding node: {:?}", why);

            why
        });

        Box::new(done)
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

        for (name, node) in &self.nodes {
            let total = node.penalty().unwrap_or(0);

            if total < record {
                best = Some(name.as_ref());
                record = total;
            }
        }

        best
    }

    /// Closes a node by websocket host.
    ///
    /// Returns whether closing the node was successful. This can fail if the
    /// node is not recognized by host.
    ///
    /// Returns `Ok(true)` if the closing was successful. Returns `Ok(false)` if
    /// it wasn't.
    pub fn close(&mut self, websocket_host: &str) -> Result<bool, Error> {
        match self.nodes.get_mut(websocket_host) {
            Some(host) => {
                host.close()?;

                Ok(true)
            },
            None => Ok(false),
        }
    }

    /// Closes all of the nodes owned by the manager.
    ///
    /// This is also automatically called when the instance is dropped.
    // **NOTE**: This can _NOT_ call `close`, as `nodes` is already mutably
    // borrowed here.
    pub fn close_all(&mut self) {
        self.nodes.values_mut().for_each(|node| {
            if let Err(why) = node.close() {
                error!("Failed to close node: {:?}", why);
            }
        });
    }

    /// Creates a new player using a [`Node`].
    ///
    /// [`Node`]: struct.Node.html
    pub fn create_player<'a>(
        &'a mut self,
        guild_id: u64,
        node_websocket_host: Option<&str>,
    ) -> Result<(), Error> {
        let node = match node_websocket_host {
            Some(host) => self.nodes.get(host).ok_or(Error::None)?,
            None => {
                self.best_node()
                    .and_then(|name| self.nodes.get(name))
                    .ok_or(Error::None)?
            },
        };

        let mut manager = self.player_manager.try_borrow_mut()?;

        manager.create(guild_id, node.user_to_node.clone()).map(|_| ())
    }

    /// Retrieves a node by websocket host.
    pub fn get_node(&self, node_websocket_host: &str) -> Option<&Node> {
        self.nodes.get(node_websocket_host)
    }

    /// Removes a player by guild ID.
    ///
    /// Returns `Ok(true)` if the player existed and was removed. Returns
    /// `Ok(false)` if the player did not exist.
    pub fn remove_player(&mut self, guild_id: &u64) -> Result<bool, Error> {
        Ok(self.player_manager.try_borrow_mut()?.remove(guild_id))
    }
}

impl Drop for NodeManager {
    /// Drops the manager, closing all nodes if possible.
    fn drop(&mut self) {
        self.close_all();
    }
}
