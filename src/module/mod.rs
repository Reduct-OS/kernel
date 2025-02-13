use limine::{modules::InternalModule, request::ModuleRequest};

#[used]
#[unsafe(link_section = ".requests")]
static MODULE_REQUEST: ModuleRequest = ModuleRequest::new().with_internal_modules(&[
    &InternalModule::new().with_path(limine::cstr!("/drv/acpid")),
    &InternalModule::new().with_path(limine::cstr!("/drv/pcid")),
]);

fn load_module(module: &&limine::file::File) {
    super::task::process::Process::create(
        unsafe { str::from_utf8_unchecked(module.path()) },
        unsafe { core::slice::from_raw_parts(module.addr() as *const u8, module.size() as usize) },
    );
}

pub fn load_all_module() {
    for module in MODULE_REQUEST.get_response().unwrap().modules() {
        load_module(module);
    }
}
