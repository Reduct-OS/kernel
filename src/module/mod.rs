use limine::{modules::InternalModule, request::ModuleRequest};

#[used]
#[unsafe(link_section = ".requests")]
static MODULE_REQUEST: ModuleRequest = ModuleRequest::new().with_internal_modules(&[
    &InternalModule::new().with_path(limine::cstr!("/drv/acpid")),
    &InternalModule::new().with_path(limine::cstr!("/drv/pcid")),
    &InternalModule::new().with_path(limine::cstr!("/drv/fbd")),
    &InternalModule::new().with_path(limine::cstr!("/drv/fsmd")),
    &InternalModule::new().with_path(limine::cstr!("/drv/nvmed")),
    &InternalModule::new().with_path(limine::cstr!("/usr/init")),
]);

fn load_module(module: &&limine::file::File) {
    super::task::process::Process::create(
        unsafe { str::from_utf8_unchecked(module.path()) },
        unsafe { core::slice::from_raw_parts(module.addr() as *const u8, module.size() as usize) },
    );
}

pub fn load_all_module() {
    for module in MODULE_REQUEST.get_response().unwrap().modules() {
        if module.path() == "/drv/acpid".as_bytes()
            || module.path() == "/drv/pcid".as_bytes()
            || module.path() == "/drv/fbd".as_bytes()
            || module.path() == "/drv/fsmd".as_bytes()
        {
            load_module(module);
        }
    }
}

pub fn load_named_module(path: &str) {
    for module in MODULE_REQUEST.get_response().unwrap().modules() {
        if module.path() == path.as_bytes() {
            load_module(module);
        }
    }
}
