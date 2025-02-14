use core::{
    sync::atomic::{AtomicUsize, Ordering},
    usize,
};

use crate::{
    fs::vfs::{inode::mount_to, pipe::PipeFS},
    ref_to_mut,
    task::process::ProcessId,
};
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use spin::Mutex;

use crate::task::get_current_process_id;

use super::{
    PATH_TO_PID, ROOT,
    user::UserFS,
    vfs::{
        inode::{FileInfo, InodeRef, InodeTy},
        stat_struct::Stat,
    },
};

static FILE_DESCRIPTOR_MANAGERS: Mutex<BTreeMap<ProcessId, Arc<FileDescriptorManager>>> =
    Mutex::new(BTreeMap::new());

pub enum OpenMode {
    Read = 0,
    Write = 1,
    ReadWrite = 2,
}

impl From<usize> for OpenMode {
    fn from(mode: usize) -> Self {
        match mode {
            0 => Self::Read,
            1 => Self::Write,
            2 => Self::ReadWrite,
            _ => panic!("Unknown open mode!!!"),
        }
    }
}

type FileDescriptor = usize;
type FileTuple = (InodeRef, OpenMode, usize);

struct FileDescriptorManager {
    file_descriptors: BTreeMap<FileDescriptor, FileTuple>,
    file_descriptor_allocator: AtomicUsize,
    cwd: Mutex<InodeRef>,
}

impl FileDescriptorManager {
    pub fn new(file_descriptors: BTreeMap<FileDescriptor, FileTuple>) -> Self {
        Self {
            file_descriptors,
            file_descriptor_allocator: AtomicUsize::new(3), // 0, 1, and 2 are reserved for stdin, stdout, and stderr
            cwd: Mutex::new(ROOT.lock().clone()),
        }
    }

    pub fn get_new_fd(&self) -> FileDescriptor {
        self.file_descriptor_allocator
            .fetch_add(1, Ordering::SeqCst)
    }

    pub fn add_inode(&self, inode: InodeRef, mode: OpenMode) -> FileDescriptor {
        let new_fd = self.get_new_fd();
        ref_to_mut(self)
            .file_descriptors
            .insert(new_fd, (inode, mode, 0));
        new_fd
    }

    pub fn change_cwd(&self, path: String) {
        if let Some(inode) = get_inode_by_path(path) {
            if inode.read().inode_type() == InodeTy::Dir {
                *self.cwd.lock() = inode;
            }
        }
    }

    pub fn get_cwd(&self) -> String {
        self.cwd.lock().read().get_path()
    }
}

fn get_file_descriptor_manager<'a>() -> Option<Arc<FileDescriptorManager>> {
    let pid = get_current_process_id();

    FILE_DESCRIPTOR_MANAGERS.lock().get_mut(&pid).cloned()
}

pub fn init_file_descriptor_manager(pid: ProcessId) {
    let mut file_descriptor_managers = FILE_DESCRIPTOR_MANAGERS.lock();
    file_descriptor_managers.insert(pid, Arc::new(FileDescriptorManager::new(BTreeMap::new())));
}

pub fn init_file_descriptor_manager_for_fork(this: ProcessId) {
    let parent_file_descriptor_manager = get_file_descriptor_manager().unwrap();
    FILE_DESCRIPTOR_MANAGERS
        .lock()
        .insert(this, parent_file_descriptor_manager.clone());
}

pub fn init_file_descriptor_manager_with_stdin_stdout(
    pid: ProcessId,
    stdin: InodeRef,
    stdout: InodeRef,
) {
    let mut file_descriptor_managers = FILE_DESCRIPTOR_MANAGERS.lock();

    let mut file_descriptors = BTreeMap::new();
    file_descriptors.insert(0, (stdin.clone(), OpenMode::Read, 0));
    file_descriptors.insert(1, (stdout.clone(), OpenMode::ReadWrite, 0));
    file_descriptors.insert(2, (stdout.clone(), OpenMode::ReadWrite, 0));

    file_descriptor_managers.insert(pid, Arc::new(FileDescriptorManager::new(file_descriptors)));
}

fn get_inode_by_path(path: String) -> Option<InodeRef> {
    let root = ROOT.lock().clone();

    let path = path.split("/");

    let node = root;

    for path_node in path {
        if path_node.len() > 0 {
            if let Some(child) = node.read().open(String::from(path_node)) {
                core::mem::drop(core::mem::replace(ref_to_mut(&node), child));
            } else {
                return None;
            }
        }
    }

    Some(node.clone())
}

pub fn kernel_open(path: String) -> Option<InodeRef> {
    get_inode_by_path(path)
}

pub fn get_inode_by_fd(file_descriptor: usize) -> Option<InodeRef> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let (inode, _, _) = current_file_descriptor_manager
        .file_descriptors
        .get(&file_descriptor)?;

    Some(inode.clone())
}

pub fn pipe(fd: &mut [FileDescriptor]) -> Option<usize> {
    assert_eq!(fd.len(), 2);

    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let pipe_inode = PipeFS::new();
    let pipe_fs_inode = kernel_open("/pipe".to_string())?;
    mount_to(
        pipe_inode.clone(),
        pipe_fs_inode.clone(),
        alloc::format!("pipe{}", get_current_process_id().0),
    );

    let read_descriptor =
        current_file_descriptor_manager.add_inode(pipe_inode.clone(), OpenMode::Read);
    let write_descriptor =
        current_file_descriptor_manager.add_inode(pipe_inode.clone(), OpenMode::Write);

    fd[0] = read_descriptor;
    fd[1] = write_descriptor;

    return Some(0);
}

pub fn open(path: String, open_mode: OpenMode) -> Option<usize> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    if path.starts_with(':') {
        let mut path = path.clone();
        let c = path.remove(0);
        assert_eq!(c, ':');
        let (fs_name, user_path) = path.split_once(':')?;

        if let Some(&pid) = PATH_TO_PID.lock().get(fs_name) {
            let inode = UserFS::new(pid);
            inode.write().when_mounted(user_path.to_string(), None);
            inode.read().open(user_path.to_string());
            let file_descriptor =
                current_file_descriptor_manager.add_inode(inode.clone(), open_mode);
            return Some(file_descriptor);
        }
    }

    let inode = if path.starts_with("/") {
        get_inode_by_path(path.clone())?
    } else {
        get_inode_by_path(alloc::format!(
            "{}{}",
            current_file_descriptor_manager.get_cwd(),
            path.clone()
        ))?
    };

    let file_descriptor = current_file_descriptor_manager.add_inode(inode, open_mode);

    Some(file_descriptor)
}

pub fn read(fd: FileDescriptor, buf: &mut [u8]) -> usize {
    let current_file_descriptor_manager = get_file_descriptor_manager();
    if let None = current_file_descriptor_manager {
        return 0;
    }
    let current_file_descriptor_manager = current_file_descriptor_manager.unwrap();

    if let Some((inode, _, offset)) = current_file_descriptor_manager.file_descriptors.get(&fd) {
        inode.read().read_at(*offset, buf)
    } else {
        0
    }
}

pub fn write(fd: FileDescriptor, buf: &[u8]) -> usize {
    if let Some(current_file_descriptor_manager) = get_file_descriptor_manager() {
        if let Some((inode, mode, offset)) =
            current_file_descriptor_manager.file_descriptors.get(&fd)
        {
            match mode {
                OpenMode::Write | OpenMode::ReadWrite => inode.read().write_at(*offset, buf),

                _ => 0,
            }
        } else {
            0
        }
    } else {
        0
    }
}

pub fn lseek(fd: FileDescriptor, offset: usize) -> Option<()> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let (_, _, old_offset) = ref_to_mut(current_file_descriptor_manager.as_ref())
        .file_descriptors
        .get_mut(&fd)?;
    *old_offset = offset;

    Some(())
}

pub fn close(fd: FileDescriptor) -> Option<()> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;
    ref_to_mut(current_file_descriptor_manager.as_ref())
        .file_descriptors
        .remove(&fd)?;
    Some(())
}

pub fn fsize(fd: FileDescriptor) -> Option<usize> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let (inode, _, _) = ref_to_mut(current_file_descriptor_manager.as_ref())
        .file_descriptors
        .get_mut(&fd)?;

    let size = inode.read().size();

    Some(size)
}

pub fn fstat(fd: FileDescriptor, buf_addr: usize) -> Option<usize> {
    let current_file_descriptor_manager = get_file_descriptor_manager()?;

    let (inode, _, _) = ref_to_mut(current_file_descriptor_manager.as_ref())
        .file_descriptors
        .get_mut(&fd)?;

    let size = inode.read().size();

    let mut stat_strcut = Stat::default();
    stat_strcut.st_size = size as u64;

    let stat_buf = &*stat_strcut;

    unsafe { core::slice::from_raw_parts_mut(buf_addr as *mut u8, stat_buf.len()) }
        .copy_from_slice(stat_buf);

    Some(0)
}

pub fn list_dir(fd: FileDescriptor) -> Vec<FileInfo> {
    if let Some(current_file_descriptor_manager) = get_file_descriptor_manager() {
        let current = current_file_descriptor_manager.get_cwd();
        if let Some(inode) = get_inode_by_fd(fd) {
            if inode.read().inode_type() == InodeTy::Dir {
                let mut list = inode.read().list();
                list.sort();

                return list;
            } else {
                return Vec::new();
            }
        } else {
            return Vec::new();
        }
    } else {
        return Vec::new();
    }
}

pub fn change_cwd(path: String) {
    if let Some(current_file_descriptor_manager) = get_file_descriptor_manager() {
        if path.starts_with("/") {
            current_file_descriptor_manager.change_cwd(path);
        } else {
            let current = current_file_descriptor_manager.get_cwd();
            let new = alloc::format!("{}{}", current, path);
            current_file_descriptor_manager.change_cwd(new);
        }
    }
}

pub fn get_cwd() -> String {
    if let Some(current_file_descriptor_manager) = get_file_descriptor_manager() {
        current_file_descriptor_manager.get_cwd()
    } else {
        String::from("/")
    }
}

pub fn create(path: String, ty: InodeTy, mode: OpenMode) -> Option<FileDescriptor> {
    if let Some(current_file_descriptor_manager) = get_file_descriptor_manager() {
        if path.starts_with("/") {
            let mut name = String::new();
            let parent_path = {
                let mut path = path.clone();
                while !path.ends_with("/") {
                    name.push(path.pop().unwrap());
                }
                path
            };
            let parent = get_inode_by_path(parent_path)?;
            let inode = parent.read().create(name.clone(), ty)?;
            let file_descriptor = current_file_descriptor_manager.add_inode(inode, mode);
            Some(file_descriptor)
        } else {
            let cwd = current_file_descriptor_manager.get_cwd();
            let parent = get_inode_by_path(cwd.clone())?;
            let inode = parent.read().create(path.clone(), ty)?;
            let file_descriptor = current_file_descriptor_manager.add_inode(inode, mode);
            Some(file_descriptor)
        }
    } else {
        None
    }
}

pub fn get_type(fd: FileDescriptor) -> Option<InodeTy> {
    if let Some(current_file_descriptor_manager) = get_file_descriptor_manager() {
        let (inode, _, _) = current_file_descriptor_manager.file_descriptors.get(&fd)?;
        Some(inode.read().inode_type())
    } else {
        None
    }
}

pub fn ioctl(fd: FileDescriptor, cmd: usize, arg: usize) -> usize {
    let inode = get_inode_by_fd(fd);
    if let Some(inode) = inode {
        return inode.read().ioctl(cmd, arg);
    }

    return usize::MAX;
}
