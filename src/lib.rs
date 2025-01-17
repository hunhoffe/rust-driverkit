#![feature(core_intrinsics, allocator_api, nonnull_slice_from_raw_parts)]
#![cfg_attr(unix, feature(libc))]
#![no_std]

#[cfg_attr(unix, macro_use)]
#[cfg(unix)]
extern crate std;

extern crate alloc;

#[cfg(unix)]
extern crate byteorder;
#[cfg(unix)]
extern crate libc;
#[cfg_attr(unix, macro_use(matches, assert_matches))]
#[cfg(unix)]
extern crate matches;

#[cfg(unix)]
extern crate mmap;

#[cfg(target_os = "barrelfish")]
extern crate libbarrelfish;

pub mod devq;
pub mod iomem;
pub mod pci;
#[cfg(unix)]
pub mod timedops;

/// Definitions for network devices.
pub mod net;

#[cfg(target_os = "barrelfish")]
mod barrelfish;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "barrelfish")]
pub use barrelfish::*;

#[cfg(target_os = "linux")]
pub use linux::*;

// The aarch64 platform specific code.
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86/mod.rs"]
mod arch;

// The aarch64 platform specific code.
#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;

pub use arch::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DriverState {
    Uninitialized,
    Initialized,
    Attached(usize),
    Detached,
    Destroyed,
}

/// Driver life-cycle management trait
pub trait DriverControl: Sized {
    /// Initialize the device
    /// DriverState must be Uninitialized
    fn init(&mut self) {
        assert!(self.state() == DriverState::Uninitialized);
        self.set_state(DriverState::Initialized);
    }

    /// Attach the driver to the device (claim ownership)
    /// DriverState must be Initialized, Detached or Attached(x)
    fn attach(&mut self) {
        #[cfg(unix)]
        assert!(
            self.state() == DriverState::Initialized
                || self.state() == DriverState::Detached
                || matches!(self.state(), DriverState::Attached(_))
        );
        self.set_state(DriverState::Attached(0));
    }

    /// Detach the driver from the device
    /// DriverState must be Detached, Attached(x)
    fn detach(&mut self) {
        #[cfg(unix)]
        assert!(matches!(self.state(), DriverState::Attached(_)));
        self.set_state(DriverState::Detached);
    }

    /// Detach the driver from the device
    /// DriverState must be Detached, Attached(x)
    fn set_sleep_level(&mut self, level: usize) {
        #[cfg(unix)]
        assert_matches!(self.state(), DriverState::Attached(_));
        self.set_state(DriverState::Attached(level));
    }

    fn destroy(mut self) {
        #[cfg(unix)]
        assert!(matches!(self.state(), DriverState::Attached(_)));
        self.set_state(DriverState::Destroyed);
    }

    fn state(&self) -> DriverState;
    fn set_state(&mut self, ds: DriverState);
}
