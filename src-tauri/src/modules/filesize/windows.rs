//! Win32 calls of `FileSize`: name and free space of a drive, and the Recycle Bin.

use std::{
    iter,
    os::windows::ffi::OsStrExt,
    path::{Component, Path, Prefix},
    ptr,
};

use windows_sys::Win32::{
    Foundation::HWND,
    Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetVolumeInformationW},
    System::WindowsProgramming::DRIVE_FIXED,
    UI::Shell::{
        FO_DELETE, FOF_ALLOWUNDO, FOF_NO_CONNECTED_ELEMENTS, FOF_NOCONFIRMATION, FOF_NOERRORUI,
        FOF_SILENT, FOF_WANTNUKEWARNING, SHFILEOPSTRUCTW, SHFileOperationW,
    },
};

use crate::app::{AppError, Result};

/// Total and free bytes of the drive holding `path`.
pub fn space(path: &Path) -> Option<(u64, u64)> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let (mut available, mut total, mut free) = (0u64, 0u64, 0u64);
    // SAFETY: `wide` is a terminated UTF-16 path and the out pointers live for the call.
    let done = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &raw mut available,
            &raw mut total,
            &raw mut free,
        )
    };
    (done != 0).then_some((total, free))
}

/// The name given to a drive ("System", "Data"), if any.
pub fn label(path: &Path) -> Option<String> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut name = [0u16; 261];
    // SAFETY: `wide` is terminated, the name buffer length is passed, other outputs are skipped.
    let done = unsafe {
        GetVolumeInformationW(
            wide.as_ptr(),
            name.as_mut_ptr(),
            261,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
        )
    };
    let length = name.iter().position(|&unit| unit == 0).unwrap_or(0);
    (done != 0 && length > 0).then(|| String::from_utf16_lossy(&name[..length]))
}

/// Only fixed drives have a Recycle Bin: on a USB stick or a network share the shell would
/// delete for good.
pub fn has_recycle_bin(path: &Path) -> bool {
    let Some(Component::Prefix(prefix)) = path.components().next() else {
        return false;
    };
    let Prefix::Disk(letter) = prefix.kind() else {
        return false;
    };
    let root: Vec<u16> = format!("{}:\\", char::from(letter))
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: `root` is a terminated UTF-16 string.
    unsafe { GetDriveTypeW(root.as_ptr()) == DRIVE_FIXED }
}

/// Sends a file or folder to the Recycle Bin. When the shell cannot (an item larger than the
/// bin), `FOF_WANTNUKEWARNING` makes it ask before deleting for good, over `owner`, and a "no"
/// cancels; without `FOF_ALLOWUNDO` it would delete straight away, so the flags are checked first.
pub fn recycle(path: &Path, owner: isize) -> Result<()> {
    let flags = u16::try_from(
        FOF_ALLOWUNDO
            | FOF_WANTNUKEWARNING
            | FOF_NO_CONNECTED_ELEMENTS
            | FOF_NOCONFIRMATION
            | FOF_NOERRORUI
            | FOF_SILENT,
    )
    .map_err(|_| AppError::Unsupported)?;
    // The shell takes a list of paths ended by an empty one, hence two terminating zeros.
    let from: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(iter::repeat_n(0, 2))
        .collect();
    let mut operation = SHFILEOPSTRUCTW {
        hwnd: owner as HWND,
        wFunc: FO_DELETE,
        pFrom: from.as_ptr(),
        pTo: ptr::null(),
        fFlags: flags,
        fAnyOperationsAborted: 0,
        hNameMappings: ptr::null_mut(),
        lpszProgressTitle: ptr::null(),
    };
    // SAFETY: `operation` points to `from`, a double-terminated path list alive for the call.
    let result = unsafe { SHFileOperationW(&raw mut operation) };
    if result == 0 && operation.fAnyOperationsAborted == 0 {
        Ok(())
    } else {
        Err(AppError::Failed(format!(
            "could not move it to the Recycle Bin ({result})"
        )))
    }
}
