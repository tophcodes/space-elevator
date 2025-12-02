pub mod config;
mod error;
mod event;
#[cfg(feature = "lcd")]
pub mod lcd;

use config::Config;
use error::{Error, Result};
use event::{ButtonEvent, Event, MotionEvent};
#[cfg(feature = "lcd")]
use lcd::{Lcd, LcdError};
use spnav_sys as ffi;
use std::mem::MaybeUninit;

/// Connection to the spacenavd daemon
///
/// This struct represents an active connection to the spacenavd daemon.
/// Only one connection per process is allowed.
///
/// The connection is automatically closed when this struct is dropped.
pub struct SpaceNav {
    _marker: std::marker::PhantomData<*mut ()>,
}

impl SpaceNav {
    /// Connect to the spacenavd daemon
    ///
    /// Opens a connection to the daemon. Only one connection per process is allowed.
    /// The daemon must be running for this to succeed.
    ///
    /// # Errors
    ///
    /// Returns `Error::ConnectionFailed` if the daemon is not running or
    /// connection fails for any other reason.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use spnav::SpaceNav;
    ///
    /// let mut device = SpaceNav::connect()?;
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn connect() -> Result<Self> {
        unsafe {
            if ffi::spnav_open() == -1 {
                return Err(Error::ConnectionFailed);
            }
        }

        Ok(Self {
            _marker: std::marker::PhantomData,
        })
    }

    /// Access configuration interface
    ///
    /// **Warning**: Configuration changes affect all applications using spacenavd.
    /// Use this primarily in configuration tools or for temporary per-application
    /// settings.
    pub fn config(&mut self) -> Config<'_> {
        Config::new(self)
    }

    #[cfg(feature = "lcd")]
    pub fn lcd(&mut self) -> std::result::Result<Lcd, LcdError> {
        Lcd::new()
    }

    /// Wait for the next space mouse event (blocking)
    ///
    /// This function blocks until an event is available from the device.
    /// Use this when you want to process events in a simple loop.
    ///
    /// # Returns
    ///
    /// Returns the next available event, or an error if something went wrong.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use spnav::{SpaceNav, Event};
    ///
    /// let mut device = SpaceNav::connect()?;
    ///
    /// loop {
    ///     match device.wait_event()? {
    ///         Event::Motion(motion) => {
    ///             println!("Motion: x={}, y={}, z={}", motion.x, motion.y, motion.z);
    ///         }
    ///         Event::Button(button) => {
    ///             println!("Button {}: {}", button.button, if button.pressed { "pressed" } else { "released" });
    ///         }
    ///     }
    /// }
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn wait_event(&mut self) -> Result<Event> {
        unsafe {
            let mut raw_event = MaybeUninit::<ffi::spnav_event>::uninit();

            if ffi::spnav_wait_event(raw_event.as_mut_ptr()) == 0 {
                return Err(Error::NoEvent);
            }

            let raw_event = raw_event.assume_init();
            Event::from_raw(raw_event)
        }
    }

    /// Check for events without blocking
    ///
    /// This function returns immediately with any available event, or `None`
    /// if no event is currently available. Use this in event loops where you
    /// need to do other work between checking for space mouse events.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(event))` - An event was available
    /// * `Ok(None)` - No event currently available
    /// * `Err(_)` - An error occurred
    ///
    /// # Example
    ///
    /// ```no_run
    /// use spnav::{SpaceNav, Event};
    ///
    /// let mut device = SpaceNav::connect()?;
    ///
    /// loop {
    ///     if let Some(event) = device.poll_event()? {
    ///         match event {
    ///             Event::Motion(motion) => {
    ///                 // Handle motion
    ///             }
    ///             Event::Button(button) => {
    ///                 // Handle button
    ///             }
    ///         }
    ///     }
    ///     
    ///     // Do other work...
    /// }
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn poll_event(&mut self) -> Result<Option<Event>> {
        unsafe {
            let mut raw_event = MaybeUninit::<ffi::spnav_event>::uninit();

            match ffi::spnav_poll_event(raw_event.as_mut_ptr()) {
                0 => Ok(None),
                _ => {
                    let raw_event = raw_event.assume_init();
                    Ok(Some(Event::from_raw(raw_event)?))
                }
            }
        }
    }

    /// Get the file descriptor for use with select/poll/epoll
    ///
    /// This returns the file descriptor that can be used with system calls
    /// like `select()`, `poll()`, or `epoll()` to wait for events without blocking.
    /// When the file descriptor becomes readable, you can call `poll_event()`
    /// to retrieve the available event.
    ///
    /// # Returns
    ///
    /// The file descriptor, or -1 if not available.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use spnav::SpaceNav;
    ///
    /// let device = SpaceNav::connect()?;
    /// let fd = device.file_descriptor();
    ///
    /// // Use fd with select/poll/epoll...
    /// # Ok::<(), spnav::Error>(())
    /// ```
    pub fn file_descriptor(&self) -> i32 {
        unsafe { ffi::spnav_fd() }
    }
}

impl Drop for SpaceNav {
    fn drop(&mut self) {
        unsafe {
            ffi::spnav_close();
        }
    }
}
