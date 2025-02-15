use alloc::{string::String, sync::Arc};
use limine::request::FramebufferRequest;
use spin::RwLock;

use crate::ref_to_mut;

use super::inode::{Inode, InodeRef};

#[used]
#[unsafe(link_section = ".requests")]
static FB_REQUEST: FramebufferRequest = FramebufferRequest::new();

pub struct FbFS {
    path: String,
    width: usize,
    height: usize,
    frame_buffer: &'static mut [u32],
}

impl FbFS {
    pub fn new() -> InodeRef {
        let fb = FB_REQUEST
            .get_response()
            .unwrap()
            .framebuffers()
            .next()
            .unwrap();

        let width = fb.width() as usize;
        let height = fb.height() as usize;

        let frame_buffer =
            unsafe { core::slice::from_raw_parts_mut(fb.addr() as *mut u32, width * height) };

        let inode = Arc::new(RwLock::new(Self {
            path: String::new(),
            width,
            height,
            frame_buffer,
        }));
        inode
    }
}

impl Inode for FbFS {
    fn when_mounted(&mut self, path: String, father: Option<InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn write_at(&self, _fd: usize, offset: usize, buf: &[u8]) -> usize {
        let frame_buffer = unsafe {
            core::slice::from_raw_parts_mut(
                ref_to_mut(self).frame_buffer.as_mut_ptr() as *mut u8,
                self.frame_buffer.len() * size_of::<u32>(),
            )
        };

        if buf.len() + offset > frame_buffer.len() {
            return 0;
        }

        frame_buffer[offset..].copy_from_slice(buf);

        return buf.len();
    }

    fn size(&self, _fd: usize) -> usize {
        self.frame_buffer.len() * size_of::<u32>()
    }

    fn inode_type(&self) -> super::inode::InodeTy {
        super::inode::InodeTy::File
    }

    fn ioctl(&self, cmd: usize, _arg: usize) -> usize {
        let ret = match cmd {
            1 => self.width,
            2 => self.height,
            _ => 0,
        };

        ret
    }
}
