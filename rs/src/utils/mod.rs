use uuid::Uuid;

pub mod user;

pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}

pub fn uuid_to_string(uuid: &Uuid) -> String {
    let mut buf = [0u8; 36];
    let s = uuid.simple().encode_lower(&mut buf);
    s.to_owned()
}
