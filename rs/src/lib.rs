pub mod application;
pub mod config;
pub mod domain;
pub mod repository;

pub type Holder<T> = Box<T>;
#[macro_export]
macro_rules! holder {
    ($t:tt) => {
        crate::Holder<dyn $t + Send + Sync>
    };
}
