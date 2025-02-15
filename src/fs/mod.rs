use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
};
use spin::{Lazy, Mutex};
use vfs::{
    acpi::AcpiFS,
    fb::FbFS,
    inode::{InodeRef, mount_to},
    root::RootFS,
};

use crate::task::process::ProcessId;

pub mod operation;
pub mod user;
pub mod vfs;

pub static ROOT: Lazy<Mutex<InodeRef>> = Lazy::new(|| Mutex::new(RootFS::new()));

pub static USER_FS_MANAGER: Mutex<BTreeMap<ProcessId, usize>> = Mutex::new(BTreeMap::new());
pub static PATH_TO_PID: Mutex<BTreeMap<String, ProcessId>> = Mutex::new(BTreeMap::new());

pub fn init() {
    ROOT.lock().write().when_mounted("/".to_string(), None);

    let dev_fs = RootFS::new();
    mount_to(dev_fs.clone(), ROOT.lock().clone(), "dev".to_string());

    let acpi_fs = AcpiFS::new();
    mount_to(acpi_fs.clone(), dev_fs.clone(), "kernel.acpi".to_string());

    let fb_fs = FbFS::new();
    mount_to(fb_fs.clone(), dev_fs.clone(), "kernel.fb".to_string());

    let pipe_fs = RootFS::new();
    mount_to(pipe_fs.clone(), ROOT.lock().clone(), "pipe".to_string());
}
