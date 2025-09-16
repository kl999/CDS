use sha2::{Digest, Sha256};
use std::fmt;

use crate::control_message::ControlMessage;

/// A message struct that contains an ID, SHA-256 hash, and data payload.
/// The hash is computed from the ID and data to ensure message integrity.
pub struct Message {
    /// Unique identifier for the message
    pub id: u64,
    /// SHA-256 hash of the ID and data combined
    pub hash: Box<[u8]>,
    /// Message payload data
    pub data: Box<[u8]>,
}

impl Message {
    /// Creates a new message with the given ID and data.
    ///
    /// The method automatically computes a SHA-256 hash of the ID and data combined.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the message
    /// * `data` - Message payload data (must be ≤ 500 bytes)
    ///
    /// # Panics
    ///
    /// Panics if the data length exceeds 500 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use udp_connection::Message;
    /// let data = b"Hello, World!".to_vec().into_boxed_slice();
    /// let message = Message::new(1, data);
    /// assert_eq!(message.id, 1);
    /// ```
    pub fn new(id: u64, data: Box<[u8]>) -> Message {
        if data.len() > 500 {
            panic!("To big packet!")
        }
        let mut hasher = Sha256::new();
        hasher.update(&id.to_be_bytes());
        hasher.update(&data);
        let hash: Box<[u8]> = hasher.finalize().to_vec().into_boxed_slice();

        Message {
            id,
            hash: hash,
            data,
        }
    }

    /// Creates a new acknowledgment message for a given message ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the message being acknowledged
    ///
    /// # Returns
    ///
    /// A new Message with:
    /// - ID set to 0 (indicating a control message)
    /// - Empty hash (32 zero bytes)
    /// - Data containing the message type (1 for ACK) and the acknowledged message ID
    ///
    /// # Examples
    ///
    /// ```
    /// # use udp_connection::Message;
    /// let ack = Message::new_acc(42);
    /// assert_eq!(ack.id, 0); // Control messages always have ID 0
    /// assert_eq!(ack.data.len(), 9); // 1 byte type + 8 bytes message ID
    /// ```
    pub fn new_acc(id: u64) -> Message {
        let hash = [0u8; 32].to_vec().into_boxed_slice();
        let data = 1u8
            .to_be_bytes()
            .iter()
            .chain(id.to_be_bytes().iter())
            .copied()
            .collect::<Vec<u8>>()
            .into_boxed_slice();
        let id = 0u64;

        Message { id, hash, data }
    }

    /// Deserializes a byte buffer into a Message.
    ///
    /// The expected format is:
    /// - Bytes 0-8: Message ID (big-endian u64)
    /// - Bytes 8-40: SHA-256 hash (32 bytes)
    /// - Bytes 40+: Message data
    ///
    /// # Arguments
    ///
    /// * `ser` - Serialized message buffer (must be ≥ 40 bytes)
    ///
    /// # Panics
    ///
    /// Panics if the buffer is less than 40 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use udp_connection::Message;
    /// let mut buffer = Vec::new();
    /// buffer.extend_from_slice(&100u64.to_be_bytes());
    /// buffer.extend_from_slice(&[0u8; 32]); // hash
    /// buffer.extend_from_slice(b"test");
    /// let message = Message::deserialize(&buffer);
    /// assert_eq!(message.id, 100);
    /// ```
    pub fn deserialize(ser: &[u8]) -> Message {
        if ser.len() < 40 {
            panic!("Buffer length < 40")
        }

        let id = u64::from_be_bytes(ser[0..8].try_into().expect("Error casting id!"));

        let hash = ser[8..40].to_vec().into_boxed_slice();

        let data = ser[40..].to_vec().into_boxed_slice();

        Message { id, hash, data }
    }

    /// Converts this message into a ControlMessage if it is a control message (id = 0).
    ///
    /// # Returns
    ///
    /// A ControlMessage variant based on the message data.
    ///
    /// # Panics
    ///
    /// * If the message ID is not 0 (not a control message)
    /// * If the control message type is unknown
    /// * If the message data is malformed
    ///
    /// # Examples
    ///
    /// ```
    /// # use udp_connection::Message;
    /// # use udp_connection::ControlMessage;
    /// // Create an acknowledgment control message
    /// let ack = Message::new_acc(123);
    /// 
    /// // Convert it to a ControlMessage
    /// let control = ack.get_control();
    /// match control {
    ///     ControlMessage::Acc { id } => assert_eq!(id, 123),
    /// }
    /// ```
    pub fn get_control(self) -> ControlMessage {
        if self.id != 0 {
            panic!("It is not control message!")
        }

        match u8::from_be_bytes((&self.data[..1]).try_into().expect("wtf?")) {
            1 => {
                let msg_id = u64::from_be_bytes((&self.data[1..9]).try_into().unwrap());
                ControlMessage::Acc { id: msg_id }
            }
            type_id => panic!("Unknown type ({})!", type_id),
        }
    }

    /// Verifies the integrity of the message by checking its hash.
    ///
    /// Recomputes the SHA-256 hash from the current ID and data,
    /// then compares it with the stored hash.
    ///
    /// # Returns
    ///
    /// `true` if the hash is valid, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use udp_connection::Message;
    /// let data = b"test data".to_vec().into_boxed_slice();
    /// let message = Message::new(42, data);
    /// assert!(message.check_hash());
    /// ```
    pub fn check_hash(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&self.id.to_be_bytes());
        hasher.update(&self.data);

        self.hash[..] == hasher.finalize()[..]
    }

    /// Serializes the message into a byte buffer.
    ///
    /// The serialized format is:
    /// - Bytes 0-8: Message ID (big-endian u64)
    /// - Bytes 8-40: SHA-256 hash (32 bytes)
    /// - Bytes 40+: Message data
    ///
    /// # Returns
    ///
    /// A boxed slice containing all message data concatenated together.
    ///
    /// # Examples
    ///
    /// ```
    /// # use udp_connection::Message;
    /// let data = b"test".to_vec().into_boxed_slice();
    /// let message = Message::new(123, data);
    /// let serialized = message.serialize();
    /// assert_eq!(serialized.len(), 8 + 32 + 4); // id + hash + data
    /// ```
    pub fn serialize(&self) -> Box<[u8]> {
        self.id
            .to_be_bytes()
            .iter()
            .chain(self.hash.iter())
            .chain(self.data.iter())
            .copied()
            .collect::<Vec<u8>>()
            .into_boxed_slice()
    }
}

/// Display implementation for Message.
///
/// Formats the message as: `#{id} ({hash:x?}): {data}`
/// where the data is displayed as a UTF-8 string (lossy conversion).
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{} ({:x?}): {}",
            self.id,
            self.hash,
            String::from_utf8_lossy(&self.data)
        )
    }
}
