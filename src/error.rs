use std::ffi::NulError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("MQTT initialization failed")]
    InitializationError,
    #[error("Invalid broker URL")]
    InvalidBrokerUrl,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Connection failed")]
    ConnectionError,
    #[error("Subscription failed")]
    SubscriptionError,
    #[error("Publication failed")]
    PublicationError,
    #[error("Invalid topic")]
    InvalidTopic,
    #[error("String contains null byte: {0}")]
    NulError(#[from] NulError),
}

pub type Result<T> = std::result::Result<T, Error>;
