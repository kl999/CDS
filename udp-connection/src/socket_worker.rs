use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    net::UdpSocket,
    rc::Rc,
};

use crate::message::Message;

pub struct SocketWorker {
    pub address: String,
    socket: UdpSocket,
    outgoing: VecDeque<Message>,
    incoming: HashMap<u64, Rc<Message>>,
    notify: fn(&[u8]),
    message_id: u64,
}

impl SocketWorker {
    pub fn new(socket: UdpSocket, address: String, f: fn(&[u8])) -> SocketWorker {
        SocketWorker {
            socket,
            address,
            outgoing: VecDeque::with_capacity(1000),
            incoming: HashMap::new(),
            notify: f,
            message_id: 1u64,
        }
    }

    pub fn work(&mut self) -> Vec<Box<[u8]>> {
        let mut msgs = Vec::new();
        loop {
            match self.receive() {
                ReceiveResult::SomeRR(msg) => {msgs.push(msg)},
                ReceiveResult::NoneRR => {break},
                _ => {}
            }
        }
        self.send();

        msgs
    }

    pub fn send_message(&mut self, msg: Box<[u8]>) {
        let msg = Message::new(self.message_id, msg);
        self.message_id += 1;
        self.outgoing.push_back(msg);
    }

    pub fn ping(){
        todo!()
    }

    fn send_acc_message(&mut self, id: u64) {
        let msg = Message::new_acc(id);
        self.outgoing.push_front(msg);
    }

    fn receive(&mut self) -> ReceiveResult {
        let mut buf = [0; 1024];
        match &self.socket.recv_from(&mut buf) {
            Ok((number_of_bytes, src_addr)) => {
                let msg = Message::deserialize(&buf[..*number_of_bytes]);
                println!(
                    "Received {} bytes from {}: C({}) '{}'",
                    number_of_bytes,
                    src_addr,
                    msg.check_hash(),
                    msg
                );

                if msg.id == 0 {
                    self.handle_ctrl(msg);
                    return ReceiveResult::Ctrl;
                }

                if !msg.check_hash() {
                    return ReceiveResult::Bad;
                }

                self.send_acc_message(msg.id);

                if self.incoming.contains_key(&msg.id) {
                    return ReceiveResult::Skip;
                }

                let msg = Rc::new(msg);

                _ = self.incoming.insert(msg.id, msg.clone());
                (self.notify)(&msg.data);

                ReceiveResult::SomeRR(msg.data.to_vec().into_boxed_slice())
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data is available right now
                ReceiveResult::NoneRR
            }
            Err(e) => {
                panic!("On receive {}", e)
            }
        }
    }

    fn send(&mut self) {
        if let Some(msg) = self.outgoing.front() {
            println!("Sending '{}'", msg);
            self.socket
                .send_to(&msg.serialize(), &self.address)
                .unwrap();
            let msg = self.outgoing.pop_front().expect("send wtf?");
            if msg.id != 0 {
                self.outgoing.push_back(msg);
            }
        }
    }

    fn handle_ctrl(&mut self, msg: Message) {
        match msg.get_control() {
            crate::control_message::ControlMessage::Acc { id } => {
                if let Some(rem_ind) = self.outgoing.iter().position(|i| i.id == id) {
                    self.outgoing.remove(rem_ind);
                }
            } /*msg => {
                  panic!("Unknown control message ({:?})", msg);
              }*/
        }
    }
}

impl Debug for SocketWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocketWorker")
            .field("socket", &self.socket)
            .field("address", &self.address)
            .field("outgoing", &self.outgoing.len())
            .field("incoming", &self.incoming.len())
            .field("notify", &self.notify)
            .field("message_id", &self.message_id)
            .finish()
    }
}

enum ReceiveResult {
    SomeRR(Box<[u8]>),
    NoneRR,
    Ctrl,
    Bad,
    Skip,
}
