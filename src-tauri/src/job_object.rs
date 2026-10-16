//! Windows Job Object z flagą `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
//!
//! Dzięki temu zamknięcie aplikacji (także awaryjne) zabija wszystkie procesy MCC
//! przypisane do tego obiektu (priorytet projektu: brak osieroconych
//! procesów). Job Object jest jeden na aplikację i żyje tak długo, jak
//! `ProcessManager`.

use std::io;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};

/// Uchwyt do obiektu zadania (Windows).
pub struct JobObject(HANDLE);

// `HANDLE` to surowy wskaźnik; Job Object jest bezpieczny do współdzielenia między
// wątkami, więc jawnie oznaczamy typ jako `Send + Sync`.
unsafe impl Send for JobObject {}
unsafe impl Sync for JobObject {}

impl JobObject {
    /// Tworzy obiekt zadania, który zabija przypisane procesy przy zamknięciu.
    pub fn new_kill_on_close() -> io::Result<Self> {
        unsafe {
            let handle = CreateJobObjectW(None, windows::core::PCWSTR::null())
                .map_err(|error| io::Error::other(error.to_string()))?;

            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                std::ptr::addr_of!(info).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(|error| {
                let _ = CloseHandle(handle);
                io::Error::other(error.to_string())
            })?;

            Ok(Self(handle))
        }
    }

    /// Przypisuje proces (surowy uchwyt) do obiektu zadania.
    pub fn assign(&self, process: HANDLE) -> io::Result<()> {
        unsafe {
            AssignProcessToJobObject(self.0, process)
                .map_err(|error| io::Error::other(error.to_string()))
        }
    }
}

impl Drop for JobObject {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
