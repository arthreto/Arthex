//! Windows shell access: file and program icons, shortcut targets. Uses COM, so it runs on worker
//! threads, never on the interface thread.

use std::{
    ffi::{OsString, c_void},
    mem,
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
    ptr,
};

use windows::{
    Win32::{
        Foundation::SIZE,
        Graphics::Gdi::{
            BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS,
            DeleteDC, DeleteObject, GetDIBits, GetObjectW, HBITMAP, HGDIOBJ,
        },
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, IPersistFile, STGM_READ,
        },
        UI::Shell::{
            BHID_EnumItems, FOLDERID_AppsFolder, IEnumShellItems, IShellItem,
            IShellItemImageFactory, IShellLinkW, KF_FLAG_DEFAULT, SHCreateItemFromParsingName,
            SHGetKnownFolderItem, SIGDN, SIGDN_NORMALDISPLAY, SIGDN_PARENTRELATIVEPARSING,
            SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY, ShellLink,
        },
    },
    core::{HSTRING, Interface},
};

fn init_com() {
    // SAFETY: initialising COM for the current thread; repeated calls are harmless.
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
}

pub type Pixels = (u32, u32, Vec<u8>);

/// The icon Windows shows for a file, program or shortcut, as RGBA pixels.
pub fn icon(path: &Path, side: i32) -> Option<Pixels> {
    init_com();
    // SAFETY: the COM object is released when dropped.
    unsafe {
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(&HSTRING::from(path), None).ok()?;
        image(&factory, side)
    }
}

pub struct ShellApp {
    pub name: String,
    /// What follows `shell:AppsFolder\` to start it: a Store app id or a program path.
    pub id: String,
    pub icon: Option<Pixels>,
}

/// Every app of the Start menu, desktop and Store alike, as `shell:AppsFolder` lists them.
pub fn apps(icon_side: i32) -> Vec<ShellApp> {
    init_com();
    let mut apps = Vec::new();
    // SAFETY: COM objects are released when dropped; names are freed by `display_name`.
    unsafe {
        let Ok(folder) =
            SHGetKnownFolderItem::<IShellItem>(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT, None)
        else {
            return apps;
        };
        let Ok(items) = folder.BindToHandler::<_, IEnumShellItems>(None, &BHID_EnumItems) else {
            return apps;
        };
        loop {
            let mut batch = [None];
            let mut fetched = 0u32;
            if items.Next(&mut batch, Some(&raw mut fetched)).is_err() || fetched == 0 {
                break;
            }
            let Some(item) = batch[0].take() else { break };
            let (Some(name), Some(id)) = (
                display_name(&item, SIGDN_NORMALDISPLAY),
                display_name(&item, SIGDN_PARENTRELATIVEPARSING),
            ) else {
                continue;
            };
            let icon = item
                .cast::<IShellItemImageFactory>()
                .ok()
                .and_then(|factory| image(&factory, icon_side));
            apps.push(ShellApp { name, id, icon });
        }
    }
    apps
}

unsafe fn display_name(item: &IShellItem, kind: SIGDN) -> Option<String> {
    // SAFETY: the shell allocates the string; it is copied, then freed with CoTaskMemFree.
    unsafe {
        let text = item.GetDisplayName(kind).ok()?;
        let value = text.to_string().ok();
        CoTaskMemFree(Some(text.0.cast_const().cast()));
        value
    }
}

unsafe fn image(factory: &IShellItemImageFactory, side: i32) -> Option<Pixels> {
    // SAFETY: the bitmap is deleted after it is read.
    unsafe {
        let bitmap = factory
            .GetImage(
                SIZE { cx: side, cy: side },
                SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK,
            )
            .ok()?;
        let pixels = read_bitmap(bitmap);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        pixels
    }
}

/// The program a `.lnk` file points to.
pub fn shortcut_target(path: &Path) -> Option<PathBuf> {
    init_com();
    // SAFETY: plain COM calls; the buffer outlives `GetPath`, which writes a terminated string.
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        link.cast::<IPersistFile>()
            .ok()?
            .Load(&HSTRING::from(path), STGM_READ)
            .ok()?;
        let mut buffer = [0u16; 1024];
        link.GetPath(&mut buffer, ptr::null_mut(), 0).ok()?;
        let length = buffer.iter().position(|&unit| unit == 0)?;
        (length > 0).then(|| PathBuf::from(OsString::from_wide(&buffer[..length])))
    }
}

/// Reads a 32-bit bitmap as straight RGBA (the shell hands out premultiplied BGRA).
unsafe fn read_bitmap(bitmap: HBITMAP) -> Option<Pixels> {
    let mut info = BITMAP::default();
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let size = mem::size_of::<BITMAP>() as i32;
    // SAFETY: `info` is a BITMAP of the announced size.
    if unsafe {
        GetObjectW(
            HGDIOBJ(bitmap.0),
            size,
            Some((&raw mut info).cast::<c_void>()),
        )
    } == 0
    {
        return None;
    }
    let width = u32::try_from(info.bmWidth).ok()?;
    let height = info.bmHeight.unsigned_abs();
    let mut header = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            #[allow(clippy::cast_possible_truncation)]
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: info.bmWidth,
            biHeight: -info.bmHeight.abs(),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut pixels = vec![0u8; usize::try_from(width * height * 4).ok()?];
    // SAFETY: the buffer holds `height` rows of `width` 32-bit pixels, as the header describes.
    let lines = unsafe {
        let dc = CreateCompatibleDC(None);
        let lines = GetDIBits(
            dc,
            bitmap,
            0,
            height,
            Some(pixels.as_mut_ptr().cast()),
            &raw mut header,
            DIB_RGB_COLORS,
        );
        let _ = DeleteDC(dc);
        lines
    };
    if lines == 0 {
        return None;
    }
    let has_alpha = pixels.as_chunks::<4>().0.iter().any(|pixel| pixel[3] != 0);
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let alpha = if has_alpha { pixel[3] } else { 255 };
        let straight = |channel: u8| -> u8 {
            if alpha == 0 || !has_alpha {
                return channel;
            }
            let value = (u16::from(channel) * 255 + u16::from(alpha) / 2) / u16::from(alpha);
            u8::try_from(value.min(255)).unwrap_or(u8::MAX)
        };
        let (blue, green, red) = (pixel[0], pixel[1], pixel[2]);
        pixel[0] = straight(red);
        pixel[1] = straight(green);
        pixel[2] = straight(blue);
        pixel[3] = alpha;
    }
    Some((width, height, pixels))
}
