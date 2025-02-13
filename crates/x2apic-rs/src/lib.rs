//! A rust interface to the x2apic interrupt architecture.

#![no_std]
#![feature(ptr_internals)]
#![allow(internal_features)]
#![deny(missing_docs)]

pub mod ioapic;
pub mod lapic;
