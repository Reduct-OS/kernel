// Copyright 2014 The syscall.rs Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This library was built for aarch64 MacOS.

use core::arch::asm;

pub mod nr;

#[inline(always)]
pub unsafe fn syscall0(n: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("x16") n,
        out("x0") ret,
        options(nostack),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall1(n: usize, a1: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("x16") n,
        inout("x0") a1 => ret,
        options(nostack),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall2(n: usize, a1: usize, a2: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("x16") n,
        inout("x0") a1 => ret,
        in("x1") a2,
        options(nostack),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("x16") n,
        inout("x0") a1 => ret,
        in("x1") a2,
        in("x2") a3,
        options(nostack),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall4(n: usize,
                       a1: usize,
                       a2: usize,
                       a3: usize,
                       a4: usize)
                       -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("x16") n,
        inout("x0") a1 => ret,
        in("x1") a2,
        in("x2") a3,
        in("x3") a4,
        options(nostack),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall5(n: usize,
                       a1: usize,
                       a2: usize,
                       a3: usize,
                       a4: usize,
                       a5: usize)
                       -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("x16") n,
        inout("x0") a1 => ret,
        in("x1") a2,
        in("x2") a3,
        in("x3") a4,
        in("x4") a5,
        options(nostack),
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall6(n: usize,
                       a1: usize,
                       a2: usize,
                       a3: usize,
                       a4: usize,
                       a5: usize,
                       a6: usize)
                       -> usize {
    let ret: usize;
    asm!(
        "svc 0",
        in("x16") n,
        inout("x0") a1 => ret,
        in("x1") a2,
        in("x2") a3,
        in("x3") a4,
        in("x4") a5,
        in("x5") a6,
        options(nostack),
    );
    ret
}
