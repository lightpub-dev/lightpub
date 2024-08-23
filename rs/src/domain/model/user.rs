use uuid::Uuid;

use super::{DateTime, URI};

// UserId value object
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UserId(Uuid);

impl UserId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn from_str(uuid: &str) -> Option<Self> {
        Uuid::parse_str(uuid).map(Self).ok()
    }

    pub fn id(&self) -> Uuid {
        self.0
    }
}

// Username value object
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Username(String);

impl Username {
    pub fn from_str(username: impl Into<String>) -> Option<Self> {
        let username = username.into();
        if Self::check(&username) {
            Some(Self(username))
        } else {
            None
        }
    }

    fn check(username: &str) -> bool {
        // username must
        // - be at least 3 characters long, and at most 16 characters long
        // - contain only alphanumeric characters + '-' + '_'
        // - not start with '-' or '_'
        // - not contain consecutive '-' or '_'

        if username.len() < 3 || username.len() > 16 {
            return false;
        }

        for ch in username.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' {
                return false;
            }
        }

        if username.starts_with('-') || username.starts_with('_') {
            return false;
        }

        let mut prev = ' ';
        for ch in username.chars() {
            if ch == '-' || ch == '_' {
                if prev == '-' || prev == '_' {
                    return false;
                }
            }
            prev = ch;
        }

        true
    }
}

// Nickname value object
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Nickname(String);

impl Nickname {
    pub fn from_str(nickname: impl Into<String>) -> Option<Self> {
        let nickname = nickname.into();
        Some(Self(nickname))
    }
}

// PrivateKey value object
#[derive(Debug, PartialEq, Clone)]
pub struct PrivateKey(String);

impl PrivateKey {
    pub fn from_str(private_key: String) -> Option<Self> {
        Some(Self(private_key))
    }
}

// PublicKey value object
#[derive(Debug, PartialEq, Clone)]
pub struct PublicKey(String);

impl PublicKey {
    pub fn from_str(public_key: String) -> Option<Self> {
        Some(Self(public_key))
    }
}

// User entity
#[derive(Debug)]
pub struct User {
    id: UserId,              // required
    username: Username,      // required
    host: Option<String>,    // null when the user is local
    bpasswd: Option<String>, // null when the user is remote
    nickname: Nickname,      // required
    bio: String,
    uri: Option<URI>,                // null when the user is local
    shared_inbox: Option<URI>,       // null when the user is local
    inbox: Option<URI>,              // null when the user is local
    outbox: Option<URI>,             // null when the user is local
    private_key: Option<PrivateKey>, // null when the user is remote
    public_key: Option<PublicKey>,
    created_at: DateTime, // required
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        // equality is based on the id
        self.id == other.id
    }
}

impl Eq for User {}

impl User {
    pub fn new(id: UserId, username: Username, nickname: Nickname, created_at: DateTime) -> Self {
        Self {
            id,
            username,
            host: None,
            bpasswd: None,
            nickname,
            bio: String::new(),
            uri: None,
            shared_inbox: None,
            inbox: None,
            outbox: None,
            private_key: None,
            public_key: None,
            created_at,
        }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn set_password(&mut self, plain_passwd: &str) -> bool {
        // hash the password
        let bpasswd = bcrypt::hash(plain_passwd, bcrypt::DEFAULT_COST).ok();
        if let Some(bpasswd) = bpasswd {
            self.bpasswd = Some(bpasswd);
            true
        } else {
            false
        }
    }

    pub fn validate_password(&self, plain_passwd: &str) -> bool {
        if let Some(bpasswd) = &self.bpasswd {
            bcrypt::verify(plain_passwd, bpasswd).unwrap_or(false)
        } else {
            false
        }
    }
}
