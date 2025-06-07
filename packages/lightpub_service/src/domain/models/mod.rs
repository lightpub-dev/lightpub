use thiserror::Error;

pub mod apub;
pub mod upload;
pub mod user;

#[derive(Debug, Clone, Error)]
pub enum IdParseError {
    #[error("parse error")]
    ParseError,
}

fn slice_to_bytes(s: &[u8]) -> [u8; 16] {
    assert_eq!(s.len(), 16);
    let mut bytes = [0; 16];
    bytes.copy_from_slice(s);
    bytes
}
