use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Failed to connect to spacenavd
    ConnectionFailed,

    /// No event available (shouldn't happen with `wait_event`)
    NoEvent,

    /// Unknown event type received
    UnknownEventType(i32),

    /// Configuration operation failed
    ConfigFailed,

    /// Invalid string (contains null bytes)
    InvalidString,

    /// Device query failed
    QueryFailed,

    /// Unknown button action code
    UnknownButtonAction(i32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConnectionFailed => {
                write!(f, "Failed to connect to spacenavd. Is it running?")
            }
            Error::NoEvent => {
                write!(f, "No event available")
            }
            Error::UnknownEventType(ty) => {
                write!(f, "Unknown event type: {}", ty)
            }
            Error::ConfigFailed => {
                write!(f, "Configuration operation failed")
            }
            Error::InvalidString => {
                write!(f, "String contains null bytes")
            }
            Error::QueryFailed => {
                write!(f, "Device query failed")
            }
            Error::UnknownButtonAction(code) => {
                write!(f, "Unknown button action code: {}", code)
            }
        }
    }
}

impl std::error::Error for Error {}
