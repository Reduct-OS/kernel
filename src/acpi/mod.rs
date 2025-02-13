use core::ptr::NonNull;

use acpi::{
    AcpiHandler, AcpiTables, HpetInfo, InterruptModel, PhysicalMapping, platform::interrupt::Apic,
};
use alloc::{alloc::Global, boxed::Box};
use limine::request::RsdpRequest;
use spin::Once;
use x86_64::{PhysAddr, VirtAddr};

use crate::memory::{convert_physical_to_virtual, convert_virtual_to_physical};

#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

pub static ACPI: Once<Box<Acpi>> = Once::new();
pub static BUF: Once<Box<[u8]>> = Once::new();

pub fn init() {
    let response = RSDP_REQUEST.get_response().unwrap();

    let acpi_tables = unsafe {
        let rsdp_address = VirtAddr::new(response.address() as u64);
        let physical_address = convert_virtual_to_physical(rsdp_address).as_u64();
        let acpi_tables = AcpiTables::from_rsdp(AcpiMemHandler, physical_address as usize);
        Box::leak(Box::new(acpi_tables.unwrap()))
    };

    log::info!("Find ACPI tables (kernel) successfully");

    let platform_info = acpi_tables
        .platform_info()
        .expect("Failed to get platform info");

    let apic = match platform_info.interrupt_model {
        InterruptModel::Apic(apic) => apic,
        InterruptModel::Unknown => panic!("No APIC support!!!"),
        _ => panic!("ACPI does not have interrupt model info!!!"),
    };

    let hpet_info = HpetInfo::new(acpi_tables).expect("Failed to get HPET info");

    let acpi = Acpi { apic, hpet_info };

    ACPI.call_once(|| Box::new(acpi));
}

pub struct Acpi<'a> {
    pub apic: Apic<'a, Global>,
    pub hpet_info: HpetInfo,
}

#[derive(Clone)]
struct AcpiMemHandler;

impl AcpiHandler for AcpiMemHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtual_address = {
            let physical_address = PhysAddr::new(physical_address as u64);
            let virtual_address = convert_physical_to_virtual(physical_address);
            NonNull::new_unchecked(virtual_address.as_u64() as *mut T)
        };
        PhysicalMapping::new(physical_address, virtual_address, size, size, self.clone())
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

pub mod apic;
pub mod hpet;
