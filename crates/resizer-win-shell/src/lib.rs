#![cfg(windows)]

use std::ffi::{OsString, c_void};
use std::fs;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows::Win32::Foundation::{
    BOOL, CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_NOTIMPL, E_OUTOFMEMORY, E_POINTER,
    HMODULE, S_FALSE, S_OK, TRUE,
};
use windows::Win32::System::Com::{
    CoTaskMemAlloc, CoTaskMemFree, IBindCtx, IClassFactory, IClassFactory_Impl,
};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::Shell::{
    IEnumExplorerCommand, IExplorerCommand, IExplorerCommand_Impl, IShellItemArray,
    SIGDN_FILESYSPATH,
};
use windows::core::{GUID, HRESULT, IUnknown, Interface, PWSTR, Result, implement};

/// Уникальный CLSID нашего расширения: {7D4B3C9E-5A1F-4E82-9B6A-3C8D1F2E4A50}
const CLSID_IMAGE_RESIZER_J: GUID = GUID::from_u128(0x7d4b3c9e_5a1f_4e82_9b6a_3c8d1f2e4a50);

static DLL_HMODULE: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_OBJECTS: AtomicUsize = AtomicUsize::new(0);
static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Выделяет UTF-16 строку в COM-куче (Проводник освободит её сам через CoTaskMemFree)
fn alloc_pwstr(text: &str) -> Result<PWSTR> {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let byte_len = wide.len() * std::mem::size_of::<u16>();
    let ptr = unsafe { CoTaskMemAlloc(byte_len) } as *mut u16;
    if ptr.is_null() {
        return Err(E_OUTOFMEMORY.into());
    }
    unsafe {
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
    }
    Ok(PWSTR(ptr))
}

/// Возвращает папку, в которой лежит сама resizer_win_shell.dll
fn dll_dir() -> Option<PathBuf> {
    let raw = DLL_HMODULE.load(Ordering::Relaxed);
    if raw == 0 {
        return None;
    }
    let hmod = HMODULE(raw as *mut c_void);
    let mut buf = [0u16; 1024];
    let len = unsafe { GetModuleFileNameW(hmod, &mut buf) } as usize;
    if len == 0 || len >= buf.len() {
        return None;
    }
    let path = PathBuf::from(OsString::from_wide(&buf[..len]));
    path.parent().map(Path::to_path_buf)
}

#[implement(IExplorerCommand)]
struct ImageResizerCommand;

impl ImageResizerCommand {
    fn new() -> Self {
        ACTIVE_OBJECTS.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

impl Drop for ImageResizerCommand {
    fn drop(&mut self) {
        ACTIVE_OBJECTS.fetch_sub(1, Ordering::Relaxed);
    }
}

impl IExplorerCommand_Impl for ImageResizerCommand_Impl {
    fn GetTitle(&self, _psiitemarray: Option<&IShellItemArray>) -> Result<PWSTR> {
        alloc_pwstr("Сжать с ImageResizerJ")
    }

    fn GetIcon(&self, _psiitemarray: Option<&IShellItemArray>) -> Result<PWSTR> {
        if let Some(dir) = dll_dir() {
            let icon_ref = format!("{},0", dir.join("imageresizej.exe").display());
            return alloc_pwstr(&icon_ref);
        }
        Err(E_NOTIMPL.into())
    }

    fn GetToolTip(&self, _psiitemarray: Option<&IShellItemArray>) -> Result<PWSTR> {
        alloc_pwstr("Уменьшить и конвертировать выделенные изображения")
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(CLSID_IMAGE_RESIZER_J)
    }

    fn GetState(
        &self,
        _psiitemarray: Option<&IShellItemArray>,
        _foktobeslow: BOOL,
    ) -> Result<u32> {
        Ok(0) // ECS_ENABLED — пункт всегда активен
    }

    fn Invoke(
        &self,
        psiitemarray: Option<&IShellItemArray>,
        _pbc: Option<&IBindCtx>,
    ) -> Result<()> {
        let Some(items) = psiitemarray else {
            return Ok(());
        };
        let count = unsafe { items.GetCount()? };
        if count == 0 {
            return Ok(());
        }

        let mut paths = Vec::with_capacity(count as usize);
        for i in 0..count {
            let item = unsafe { items.GetItemAt(i)? };
            if let Ok(pwstr) = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) } {
                let wide = unsafe { pwstr.as_wide() };
                let path = PathBuf::from(OsString::from_wide(wide));
                unsafe { CoTaskMemFree(Some(pwstr.0 as *const c_void)) };
                paths.push(path);
            }
        }

        if paths.is_empty() {
            return Ok(());
        }

        let Some(dir) = dll_dir() else {
            return Ok(());
        };
        let worker = dir.join("resizer-worker.exe");
        if !worker.is_file() {
            return Ok(());
        }

        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let list_file = std::env::temp_dir().join(format!(
            "imageresizej_batch_{}_{}.txt",
            std::process::id(),
            id
        ));

        let payload: String = paths
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");

        if fs::write(&list_file, payload).is_ok() {
            let _ = Command::new(&worker)
                .arg("--from-file")
                .arg(&list_file)
                .spawn();
        }

        Ok(())
    }

    fn GetFlags(&self) -> Result<u32> {
        Ok(0) // ECF_DEFAULT — одиночный пункт без подменю
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        Err(E_NOTIMPL.into())
    }
}

#[implement(IClassFactory)]
struct ClassFactory;

impl ClassFactory {
    fn new() -> Self {
        ACTIVE_OBJECTS.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

impl Drop for ClassFactory {
    fn drop(&mut self) {
        ACTIVE_OBJECTS.fetch_sub(1, Ordering::Relaxed);
    }
}

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Option<&IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut c_void,
    ) -> Result<()> {
        if ppvobject.is_null() {
            return Err(E_POINTER.into());
        }
        unsafe { *ppvobject = std::ptr::null_mut() };

        if punkouter.is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }

        let cmd: IExplorerCommand = ImageResizerCommand::new().into();
        unsafe { cmd.query(riid, ppvobject).ok() }
    }

    fn LockServer(&self, flock: BOOL) -> Result<()> {
        if flock.as_bool() {
            ACTIVE_OBJECTS.fetch_add(1, Ordering::Relaxed);
        } else {
            ACTIVE_OBJECTS.fetch_sub(1, Ordering::Relaxed);
        }
        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(hinst: HMODULE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        DLL_HMODULE.store(hinst.0 as usize, Ordering::Relaxed);
    }
    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    unsafe { *ppv = std::ptr::null_mut() };

    if rclsid.is_null() || unsafe { *rclsid } != CLSID_IMAGE_RESIZER_J {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    let factory: IClassFactory = ClassFactory::new().into();
    unsafe { factory.query(riid, ppv) }
}

#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if ACTIVE_OBJECTS.load(Ordering::Relaxed) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}