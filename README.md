win-fs-redirect
===============
[![Build Status](https://github.com/rustysec/win-fs-redirect-rs/workflows/Build/badge.svg)](https://github.com/rustysec/win-fs-redirect-rs/actions)

A very simple wrapper to help with Wow64 file system redirection.

# Problem
64 bit versions of Microsoft Windows implement a set of capabilities referred to as
"Windows on Windows" or WoW. This enables 32bit applications to operate natively 
on top of the 64 operating system. One of the features of Wow64 is "file sytem
redirection." What this does is hot patch 32bit applications trying to access certain
paths for their architecture specific path. An example is a 32bit version of `notepad.exe`
will attempt to load `c:\windows\system32\kernel32.dll`, however this is a 64bit library
and will not work in the 32bit application. Windows takes care of this problem by "redirecting"
the request to `c:\windows\syswow64\kernel32.dll` instead.

This can be problematic when attempting to access certain files, as the operating system 
will actually give your application a handle to a _different_ file than you expect.

This little library wraps over the `Wow64DisableWow64FsRedirection` class of APIs for an 
ergonomic way to overcome this behavior.

# Example
```rust
use win_fs_redirect::DisableFsRedirection;

fn main() {
    let s = std::fs::metadata("c:\\windows\\system32\\kernel32.dll").unwrap();
    println!("- file size: {}", s.len());
    DisableFsRedirection::start()
        .map(|_i| {
            let s = std::fs::metadata("c:\\windows\\system32\\kernel32.dll").unwrap();
            println!("+ file size: {}", s.len());
        })
        .map_err(|e| println!("Can't disable redirection: {}", e))
        .unwrap();
    let s = std::fs::metadata("c:\\windows\\system32\\kernel32.dll").unwrap();
    println!("- file size: {}", s.len());
}
```

The output of this is something along the lines of:
```
- file size: 649064
+ file size: 725696
- file size: 649064
```
(file sizes will differ on your system)

Notice here the line with the `+` is the only one inside the block with redirection disabled.
