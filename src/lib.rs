//! win-fs-redirect
//! ===============
//! A very simple wrapper to help with Wow64 filesystem redirection.
//!
//! # Problem
//! 64 bit versions of Microsoft Windows implement a set of capabilities referred to as
//! "Windows on Windows" or WoW. This enables 32bit applications to operate natively
//! on top of the 64 operating system. One of the features of Wow64 is "file sytem
//! redirection." What this does is hot patch 32bit applications trying to access certain
//! paths for their architecture specific path. An example is a 32bit version of `notepad.exe`
//! will attempt to load `c:\windows\system32\kernel32.dll`, however this is a 64bit library
//! and will not work in the 32bit application. Windows takes care of this problem by "redirecting"
//! the request to `c:\windows\syswow64\kernel32.dll` instead.
//!
//! This can be problematic when attempting to access certain files, as the operating system
//! will actually give your application a handle to a _different_ file than you expect.
//!
//! This little library wraps over the `Wow64DisableWow64FsRedirection` class of APIs for an
//! ergonomic way to overcome this behavior.
//!
//! # Example
//!
//! ```no_run
//! use win_fs_redirect::DisableFsRedirection;
//!
//! fn main() {
//!     let s = std::fs::metadata("c:\\windows\\system32\\kernel32.dll").unwrap();
//!     println!("- file size: {}", s.len());
//!     DisableFsRedirection::start()
//!         .map(|_i| {
//!             let s = std::fs::metadata("c:\\windows\\system32\\kernel32.dll").unwrap();
//!             println!("+ file size: {}", s.len());
//!         })
//!         .map_err(|e| println!("Can't disable redirection: {}", e))
//!         .unwrap();
//!     let s = std::fs::metadata("c:\\windows\\system32\\kernel32.dll").unwrap();
//!     println!("- file size: {}", s.len());
//! }
//! ```
//!
//! The output of this is something along the lines of:
//!
//! ```ignore
//! - file size: 649064
//! + file size: 725696
//! - file size: 649064
//! ```
//!
//! (file sizes will differ on your system)
//!
//! Notice here the line with the `+` is the only one inside the block with redirection disabled.

#![cfg(windows)]

#[macro_use]
extern crate log;

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winnt::PVOID;
use winapi::um::wow64apiset::{Wow64DisableWow64FsRedirection, Wow64RevertWow64FsRedirection};

/// Wrapper around pointer to file system redirection state
pub struct DisableFsRedirection(Option<*mut PVOID>);

impl DisableFsRedirection {
    /// Returns a `Result` containing either a `DisableFsRedirection` or
    /// an `Error<u32>` with the error code from Windows.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// DisableFsRedirection::start().map(|_| {
    ///     // access normally redirected files
    /// });
    /// ```
    pub fn start() -> Result<DisableFsRedirection, u32> {
        let mut old: PVOID = unsafe { std::mem::zeroed() };
        match unsafe { Wow64DisableWow64FsRedirection(&mut old) } {
            1 => Ok(DisableFsRedirection(Some(&mut old))),
            _ => Err(unsafe { GetLastError() }),
        }
    }
}

impl Drop for DisableFsRedirection {
    fn drop(&mut self) {
        if let Some(h) = self.0 {
            if unsafe { Wow64RevertWow64FsRedirection(*h) } != 1 {
                error!("Revert of file system redirection failed with {}", unsafe {
                    GetLastError()
                });
            }
        }
    }
}

#[cfg(all(test, windows, target_pointer_width = "32"))]
mod tests {
    #[test]
    fn kernel32_size() {
        use crate::DisableFsRedirection;
        let s = std::fs::metadata("c:\\windows\\system32\\kernel32.dll")
            .unwrap()
            .len();
        DisableFsRedirection::start()
            .map(|_| {
                let s1 = std::fs::metadata("c:\\windows\\system32\\kernel32.dll")
                    .unwrap()
                    .len();
                assert!(s1 != s);
            })
            .map_err(|_| assert!(false))
            .unwrap();
    }
}
