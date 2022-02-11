// the improper_ctypes is needed as because
// spdk_nvme_ctrlr_data is 128 bit

#![allow(
    clippy::all,
    elided_lifetimes_in_paths,
    improper_ctypes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unknown_lints,
    unused,
    clippy::upper_case_acronyms
)]

use std::os::raw::c_char;
include!(concat!(env!("OUT_DIR"), "/libspdk.rs"));

#[macro_use]
extern crate tracing;
extern crate serde;
extern crate serde_json;

pub mod cpu_cores;
pub mod ffihelper;
pub mod libspdk;

mod bdev;
mod bdev_async;
mod bdev_builder;
mod bdev_desc;
mod bdev_io;
mod bdev_iter;
mod bdev_module;
mod bdev_ops;
mod dma;
mod error;
mod io_channel;
mod io_device_traverse;
mod io_devices;
mod io_type;
mod json_write_context;
mod nvme;
mod poller;
mod thread;
mod untyped_bdev;
mod uuid;

pub use crate::{
    bdev::Bdev,
    bdev_async::{BdevAsyncCallContext, BdevStats},
    bdev_builder::BdevBuilder,
    bdev_desc::{BdevDesc, BdevEvent},
    bdev_io::{BdevIo, IoVec},
    bdev_iter::{BdevGlobalIter, BdevModuleIter},
    bdev_module::{
        BdevModule,
        BdevModuleBuild,
        BdevModuleBuilder,
        WithModuleConfigJson,
        WithModuleFini,
        WithModuleGetCtxSize,
        WithModuleInit,
    },
    bdev_ops::BdevOps,
    dma::{DmaBuf, DmaError},
    error::{SpdkError, SpdkResult},
    io_channel::IoChannel,
    io_device_traverse::{ChannelTraverseStatus, IoDeviceChannelTraverse},
    io_devices::IoDevice,
    io_type::{IoStatus, IoType},
    json_write_context::JsonWriteContext,
    nvme::{
        nvme_admin_opc,
        nvme_nvm_opcode,
        nvme_reservation_acquire_action,
        nvme_reservation_register_action,
        nvme_reservation_register_cptpl,
        nvme_reservation_type,
        GenericStatusCode,
        NvmeCommandStatus,
        NvmeStatus,
    },
    poller::{Poller, PollerBuilder},
    thread::Thread,
    untyped_bdev::UntypedBdev,
    uuid::Uuid,
};

/// TODO
pub type LogProto = Option<
    extern "C" fn(
        level: i32,
        file: *const c_char,
        line: u32,
        func: *const c_char,
        buf: *const c_char,
        n: i32,
    ),
>;

#[cfg(target_arch = "x86_64")]
extern "C" {
    /// TODO
    pub fn spdk_rs_log(
        level: i32,
        file: *const c_char,
        line: i32,
        func: *const c_char,
        format: *const c_char,
        args: *mut libspdk::__va_list_tag,
    );

    /// TODO
    pub static mut logfn: LogProto;
}

#[cfg(target_arch = "aarch64")]
extern "C" {
    /// TODO
    pub fn spdk_rs_log(
        level: i32,
        file: *const c_char,
        line: i32,
        func: *const c_char,
        format: *const c_char,
        args: libspdk::__va_list,
    );

    /// TODO
    pub static mut logfn: LogProto;
}
