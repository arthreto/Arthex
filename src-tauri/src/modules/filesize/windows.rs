//! Win32 calls of `FileSize`: free space of a drive and the Recycle Bin.

use std::{iter, os::windows::ffi::OsStrExt, path::Path, ptr};

use windows_sys::Win32::{
    Storage::FileSystem::GetDiskFreeSpaceExW,
    UI::Shell::{
        FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, SHFILEOPSTRUCTW,
        SHFileOperationW,
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

/// Sends a file or folder to the Recycle Bin, never deleting it for good: without
/// `FOF_ALLOWUNDO` the shell would delete it permanently, so the flags are checked first.
pub fn recycle(path: &Path) -> Result<()> {
    let flags = u16::try_from(FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT)
        .map_err(|_| AppError::Unsupported)?;
    // The shell takes a list of paths ended by an empty one, hence two terminating zeros.
    let from: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(iter::repeat_n(0, 2))
        .collect();
    let mut operation = SHFILEOPSTRUCTW {
        hwnd: ptr::null_mut(),
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
