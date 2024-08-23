use crate::application::service::user::{UserApplicationService, UserSecurityApplicationService};

pub mod model {
    use uuid::fmt::Simple;

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
        user_id Simple
    );

    // User login
    Request!(LoginRequest,
        username String,
        password String
    );
    Response!(LoginResponse,
        token String
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
}
