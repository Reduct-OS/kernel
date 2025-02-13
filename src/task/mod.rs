pub mod context;
pub mod process;
pub mod scheduler;
pub mod stack;
pub mod thread;
pub mod timer;

use process::{ProcessId, SharedProcess};
use scheduler::SCHEDULER;
use thread::SharedThread;

pub use self::scheduler::init;

pub fn get_current_thread() -> SharedThread {
    SCHEDULER.lock().current().upgrade().unwrap()
}

pub fn get_current_process() -> SharedProcess {
    get_current_thread().read().process.upgrade().unwrap()
}

pub fn get_current_process_id() -> ProcessId {
    get_current_process().read().id
}
