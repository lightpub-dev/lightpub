use crate::application::service::{
    post::PostCreateApplicationService,
    user::{UserApplicationService, UserSecurityApplicationService},
};

pub mod model {
    use uuid::Uuid;

    macro_rules! Request {
        ($name:ident, $($field:ident $type:ty),*) => {
            #[derive(Debug, serde::Deserialize)]
            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }
        }
    }

    macro_rules! Response {
        ($name:ident, $($field:ident $type:ty),*) => {
            #[derive(Debug, serde::Serialize)]
            pub struct $name {
                $(
                    pub $field: $type,
                )*
            }
        }
    }

    // User registration
    Request!(RegisterRequest,
        username String,
        nickname String,
        password String
    );
    Response!(RegisterResponse,
        user_id String
    );

    // User login
    Request!(LoginRequest,
        username String,
        password String
    );
    Response!(LoginResponse,
        token String
    );

    // Post create
    Request!(PostCreateRequest,
        content Option<String>,
        privacy String,
        reply_to_id Option<String>,
        repost_of_id Option<String>
    );

    Response!(PostCreateResponse,
        post_id String
    );
}

pub struct AppState {}

impl AppState {
    pub fn user_service(&self) -> UserApplicationService {
        todo!()
    }

    pub fn user_security_service(&self) -> UserSecurityApplicationService {
        todo!()
    }

    pub fn post_create_service(&self) -> PostCreateApplicationService {
        todo!()
    }
}
