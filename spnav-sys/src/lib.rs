#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Raw FFI bindings to `libspnav`
//!
//! This crate provides unsafe bindings to the libspnav library for
//! communicating with spacenavd to receive input from 6DOF devices.
//!
//! For a safe Rust API, see the `spnav` crate instead.
//!
//! # Requirements
//!
//! - libspnav development headers must be installed
//! - spacenavd daemon should be running to receive events
//!
//! # Example
//!
//! ```no_run
//! use spnav_sys::*;
//!
//! unsafe {
//!     if spnav_open() == -1 {
//!         panic!("Failed to connect to spacenavd");
//!     }
//!     
//!     let mut event: spnav_event = std::mem::zeroed();
//!     spnav_wait_event(&mut event);
//!     
//!     spnav_close();
//! }
//! ```

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_size() {
        assert_eq!(
            std::mem::size_of::<spnav_event>(),
            std::mem::size_of::<spnav_event_motion>()
                .max(std::mem::size_of::<spnav_event_button>())
        );
    }

    #[test]
    fn test_constants() {
        assert!(SPNAV_EVENT_MOTION > 0);
        assert!(SPNAV_EVENT_BUTTON > 0);
    }
}
