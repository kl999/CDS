use super::*;

#[test]
fn test_new() {
    let data = b"Hello, World!".to_vec().into_boxed_slice();
    let message = Message::new(1, data.clone());
    
    assert_eq!(message.id, 1);
    assert_eq!(message.data, data);
    assert_eq!(message.hash.len(), 32); // SHA-256 produces 32 bytes
}

#[test]
fn test_deserialize() {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&100u64.to_be_bytes());
    buffer.extend_from_slice(&[0u8; 32]); // dummy hash
    buffer.extend_from_slice(b"test");
    
    let message = Message::deserialize(&buffer);
    
    assert_eq!(message.id, 100);
    assert_eq!(message.hash.len(), 32);
    assert_eq!(message.data, b"test".to_vec().into_boxed_slice());
}

#[test]
fn test_check_hash() {
    let data = b"test data".to_vec().into_boxed_slice();
    let message = Message::new(42, data);
    
    assert!(message.check_hash());
}

#[test]
fn test_display() {
    let data = b"Hello".to_vec().into_boxed_slice();
    let message = Message::new(1, data);
    
    let display_string = format!("{}", message);
    assert!(display_string.contains("#1"));
    assert!(display_string.contains("Hello"));
}

#[test]
fn test_serialize_format() {
    let data = b"test".to_vec().into_boxed_slice();
    let message = Message::new(0x123456789ABCDEF0, data);
    let serialized = message.serialize();
    
    // Check total length: 8 (id) + 32 (hash) + 4 (data) = 44 bytes
    assert_eq!(serialized.len(), 44);
    
    // Check ID bytes (big-endian)
    let id_bytes = &serialized[0..8];
    assert_eq!(u64::from_be_bytes(id_bytes.try_into().unwrap()), 0x123456789ABCDEF0);
    
    // Check hash bytes
    let hash_bytes = &serialized[8..40];
    assert_eq!(hash_bytes.len(), 32);
    
    // Check data bytes
    let data_bytes = &serialized[40..];
    assert_eq!(data_bytes, b"test");
}

#[test]
fn test_serialize_deserialize_roundtrip() {
    let original_data = b"Round trip test data".to_vec().into_boxed_slice();
    let original_message = Message::new(999, original_data);
    
    // Serialize the message
    let serialized = original_message.serialize();
    
    // Deserialize it back
    let deserialized_message = Message::deserialize(&serialized);
    
    // Verify all fields match
    assert_eq!(original_message.id, deserialized_message.id);
    assert_eq!(original_message.hash.len(), deserialized_message.hash.len());
    assert_eq!(&original_message.hash[..], &deserialized_message.hash[..]);
    assert_eq!(&original_message.data[..], &deserialized_message.data[..]);
    
    // Verify hash is still valid after deserialization
    assert!(deserialized_message.check_hash());
}

#[test]
fn test_serialize_empty_data() {
    let data = Vec::new().into_boxed_slice();
    let message = Message::new(0, data);
    
    let serialized = message.serialize();
    assert_eq!(serialized.len(), 40); // 8 + 32 + 0
    
    let deserialized = Message::deserialize(&serialized);
    assert_eq!(deserialized.data.len(), 0);
    assert!(deserialized.check_hash());
}

#[test]
fn test_serialize_max_size_data() {
    let data = vec![0x42u8; 500].into_boxed_slice();
    let message = Message::new(1, data);
    
    let serialized = message.serialize();
    assert_eq!(serialized.len(), 540); // 8 + 32 + 500
    
    // Verify data section contains the expected pattern
    let data_bytes = &serialized[40..];
    assert_eq!(data_bytes.len(), 500);
    assert!(data_bytes.iter().all(|&b| b == 0x42));
}
