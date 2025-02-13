use alloc::string::ToString;
use spin::{Lazy, Mutex};
use vfs::{
    acpi::AcpiFS,
    inode::{InodeRef, mount_to},
    root::RootFS,
};

pub mod operation;
pub mod vfs;

pub static ROOT: Lazy<Mutex<InodeRef>> = Lazy::new(|| Mutex::new(RootFS::new()));

pub fn init() {
    ROOT.lock().write().when_mounted("/".to_string(), None);

    let dev_fs = RootFS::new();
    mount_to(dev_fs.clone(), ROOT.lock().clone(), "dev".to_string());

    let acpi_fs = AcpiFS::new();
    mount_to(acpi_fs.clone(), dev_fs.clone(), "kernel.acpi".to_string());

    let pipe_fs = RootFS::new();
    mount_to(pipe_fs.clone(), ROOT.lock().clone(), "pipe".to_string());
}
