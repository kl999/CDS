use std::{
    collections::{HashMap, VecDeque}, net::UdpSocket, sync::{
        Arc, Mutex,
        mpsc::{Receiver, TryRecvError},
    }
};

use udp_connection::{receive_handshake, socket_worker_handshake::receive_handshake_nonblocking};

use crate::peer::{Peer, PeerResult};

pub struct CdsWorker {
    pub client_id: u32,
    peer_map: Vec<PeerMapItem>,
    collection: Arc<Mutex<HashMap<String, Cell>>>,
    peers: Vec<Peer>,
    rx: Receiver<(String, String)>,
    new_peer_socket: UdpSocket,
}

impl CdsWorker {
    pub fn new(
        client_id: u32,
        collection: Arc<Mutex<HashMap<String, Cell>>>,
        rx: Receiver<(String, String)>,
        address: String
    ) -> Result<CdsWorker, String> {
        let new_peer_socket = UdpSocket::bind(&address)
            .map_err(|e| format!("Error binding socket {e}"))?;
        new_peer_socket.set_nonblocking(true)
            .map_err(|e| format!("Error set nonblocking {e}"))?;

        Ok(CdsWorker {
            client_id,
            peer_map: vec![],
            collection,
            peers: vec![],
            rx,
            new_peer_socket,
        })
    }

    pub fn set_key_foreign(
        &self,
        key: String,
        val: String,
        client_id: u32,
        version: u64,
    ) -> Result<(), String> {
        let mut collection = self
            .collection
            .lock()
            .map_err(|x| format!("col lock!\n{}", x))?;

        let cell = collection.get_mut(&key);

        if let Some(cell) = cell {
            if version > cell.version || version == cell.version && client_id < self.client_id {
                cell.client_id = client_id;
                cell.version = version;
                cell.value = val;
            }
            //else version < cell.version do nothing
        } else {
            collection.insert(
                key,
                Cell {
                    client_id,
                    version,
                    value: val,
                },
            );
        }

        Ok(())
    }

    //fn define_client_id(&mut self){}
    // Workflow:
    // 1. Connect to ANY bootstrap peer
    // 2. Request peer list
    // 3. Find peer with highest ID
    // 4. Connect to that peer
    // 5. Request ID assignment
    // 6. Get new_id = max_id + 1
    // 7. Announce yourself to all peers
    // 8. You are now the "last" peer (until someone else joins)

    pub fn work(mut self) {
        loop {
            if let Err(e) = self.regenerate_peers() {
                eprintln!("Push error: {}", e);
            }

            if let Err(e) = self.push_keys_to_peers() {
                eprintln!("Push error: {}", e);
            }

            let mut i = 0;
            loop {
                if i == self.peers.len() {
                    break;
                }

                let result = self.peers[i].work();

                match result {
                    Ok(result) => {
                        if let Err(msg) = self.consume_peer_result(result) {
                            eprintln!("Consume result error {}", msg);
                        }
                    }
                    Err(e) => eprintln!("Peer work error: {}", e),
                }

                i += 1;
            }
        }
    }

    fn consume_peer_result(&mut self, result: Vec<PeerResult>) -> Result<(), String> {
        for result in result {
            match result {
                PeerResult::KeyUpdate(key, value, version, client_id) => {
                    self.set_key_foreign(key, value, client_id, version)?;
                }
                _ => return Err(format!("Unknown result {:?}", result)),
            }
        }

        Ok(())
    }

    fn push_keys_to_peers(&mut self) -> Result<(), String> {
        loop {
            match self.rx.try_recv() {
                Ok((key, value)) => {
                    let mut collection = self
                        .collection
                        .lock()
                        .map_err(|x| format!("col lock!\n{}", x))?;

                    let cell = collection.get_mut(&key);
                    let mut ver = 0;

                    if let Some(cell) = cell {
                        cell.client_id = self.client_id;
                        cell.version = cell.version + 1;
                        cell.value = value.clone();
                        ver = cell.version;
                    } else {
                        collection.insert(key.clone(), Cell::new(self.client_id, value.clone()));
                    }

                    for peer in &mut self.peers {
                        peer.push_val(key.clone(), value.clone(), self.client_id, ver)?;
                    }
                }
                Err(err) if err == TryRecvError::Empty => break,
                // TODO: TryRecvError::Disconnected kill the worker!
                Err(err) => eprintln!("Error channel {}", err),
            }
        }

        Ok(())
    }

    fn regenerate_peers(&mut self) -> Result<(), String> {
        self.accept_new_peer()?;
        self.regenerate_from_map()?;

        Ok(())
    }

    fn accept_new_peer(&mut self) -> Result<(), String> {
        let worker = receive_handshake_nonblocking(
        &self.new_peer_socket,
        |_| ())
            .map_err(|x| format!("col lock!\n{}", x))?;

        let address = worker.address.clone();

        if self.dont_have_peer_with_addr(&address) {
            self.peers.push(Peer::new(address, worker));
        }

        Ok(())
    }
    fn regenerate_from_map(&self) -> Result<(), String> {
        for item in &self.peer_map {
            // TODO: if peer not found make new from map
        }

        Ok(())
    }

    pub fn dont_have_peer_with_addr(&self, address: &str) -> bool {
        !self.peers.iter().any(|peer| peer.address == address)
    }
}

pub struct Cell {
    pub client_id: u32,
    pub version: u64,
    pub value: String,
}

impl Cell {
    pub fn new(client_id: u32, value: String) -> Cell {
        Cell {
            client_id,
            version: 0,
            value,
        }
    }
}

struct PeerMapItem {
    pub address: String,
    pub client_id: u32,
    state: PeerMapState
}

impl PeerMapItem {
    pub fn new(address: String, client_id: u32) -> PeerMapItem {
        PeerMapItem {
            address,
            client_id,
            state: PeerMapState::Ok,
        }
    }
}

enum PeerMapState {
    Unknown,
    Ok,
    Inactive
}
