extern crate bindgen;
extern crate cc;

use crate::build_helpers::{print_path_var, Error, LibraryConfig};
use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};
use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

mod build_helpers;

#[derive(Debug)]
struct MacroCallback {
    macros: Arc<RwLock<HashSet<String>>>,
}

impl ParseCallbacks for MacroCallback {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        self.macros.write().unwrap().insert(name.into());

        if name == "IPPORT_RESERVED" {
            return MacroParsingBehavior::Ignore;
        }

        MacroParsingBehavior::Default
    }
}

/// Returns package's root dir.
fn get_root_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
}

/// Returns output dir.
fn get_out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

/// Returns target dir.
fn get_target_dir() -> PathBuf {
    let mut p = get_out_dir();
    p.pop();
    p.pop();
    assert_eq!(p.file_name(), Some(OsStr::new("build")));
    p.pop();
    p
}

/// Returns absolute path for SPDK library.
fn get_spdk_path() -> Result<PathBuf, Error> {
    let spdk_path = match env::var_os("SPDK_PATH") {
        Some(s) => {
            println!("SPDK_PATH variable is set to {}", s.to_str().unwrap());
            PathBuf::from(s)
        }
        None => {
            let mut spdk_path = get_root_dir();
            spdk_path.push("spdk");
            println!(
                "SPDK_PATH variable not set, trying {}",
                spdk_path.to_str().unwrap()
            );
            spdk_path
        }
    };

    match fs::canonicalize(&spdk_path) {
        Ok(res) => {
            println!("SPDK found at {}", res.to_str().unwrap());
            Ok(res)
        }
        Err(e) => Err(Error::Generic(format!(
            "Bad SPDK path {}: {}",
            spdk_path.to_str().unwrap(),
            e
        ))),
    }
}

/// Finds and configures SPDK library.
fn configure_spdk() -> Result<LibraryConfig, Error> {
    let spdk_path = get_spdk_path()?;

    print_path_var("****", "PKG_CONFIG_PATH");
    print_path_var("****", "PKG_CONFIG_PATH_FOR_TARGET");

    let mut spdk_lib = LibraryConfig::new();

    spdk_lib.add_inc(spdk_path.join("include"))?;
    spdk_lib.add_inc(spdk_path.join("include/spdk_internal"))?;

    spdk_lib.add_inc_alt(
        spdk_path.join("include/spdk/module"),
        spdk_path.join("module"),
    )?;
    spdk_lib.add_inc_alt(
        spdk_path.join("include/spdk/lib"),
        spdk_path.join("lib"),
    )?;

    spdk_lib.find_pkg_config_dirs(&spdk_path)?;

    spdk_lib.exclude_lib("spdk_bdev_blobfs");
    spdk_lib.exclude_lib("spdk_bdev_ftl");
    spdk_lib.exclude_lib("spdk_bdev_gpt");
    spdk_lib.exclude_lib("spdk_bdev_passthru");
    spdk_lib.exclude_lib("spdk_bdev_raid");
    spdk_lib.exclude_lib("spdk_bdev_split");
    spdk_lib.exclude_lib("spdk_bdev_zone_block");
    spdk_lib.exclude_lib("spdk_event_nvmf");
    spdk_lib.exclude_lib("spdk_sock_uring");
    spdk_lib.exclude_lib("spdk_ut_mock");

    spdk_lib.mark_system("aio");
    spdk_lib.mark_system("bsd");
    spdk_lib.mark_system("crypto");
    spdk_lib.mark_system("dl");
    spdk_lib.mark_system("m");
    spdk_lib.mark_system("numa");
    spdk_lib.mark_system("pcap");
    spdk_lib.mark_system("rt");
    spdk_lib.mark_system("uring");
    spdk_lib.mark_system("uuid");

    spdk_lib.set_static_search(true);

    spdk_lib.find_lib("libdpdk")?;

    spdk_lib.find_libs(&vec![
        "spdk_accel",
        "spdk_accel_ioat",
        "spdk_bdev_aio",
        "spdk_bdev_crypto",
        "spdk_bdev_delay",
        "spdk_bdev_error",
        "spdk_bdev_lvol",
        "spdk_bdev_malloc",
        "spdk_bdev_null",
        "spdk_bdev_nvme",
        "spdk_bdev_uring",
        "spdk_bdev_virtio",
        "spdk_env_dpdk",
        "spdk_env_dpdk_rpc",
        "spdk_event",
        "spdk_event_accel",
        "spdk_event_bdev",
        "spdk_event_iscsi",
        "spdk_event_nbd",
        "spdk_event_scsi",
        "spdk_event_sock",
        "spdk_event_vmd",
        "spdk_nvmf",
    ])?;

    spdk_lib.find_lib("spdk_syslibs")?;

    spdk_lib.dump();

    println!("Merging SPDK static libraries into a shared library...");
    let lib_name = OsStr::new("spdk-bundle");
    let lib_dir = get_target_dir();
    let lib_path = spdk_lib.build_shared_lib(&lib_dir, lib_name)?;

    println!("cargo:rustc-link-lib=dylib={}", lib_name.to_str().unwrap());
    println!("cargo:root={}", lib_dir.to_str().unwrap());
    println!("cargo:lib_path={}", lib_path.to_str().unwrap());

    // TODO: linking to static SPDK requries `+whole-archive` option
    // TODO: which is currently supported only by a nightly compiler:
    // TODO: "-Z", "unstable-options",
    //println!("Link against static SPDK...");
    //spdk_lib.cargo();

    println!("cargo:rerun-if-env-changed=SPDK_PATH");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH_FOR_TARGET");

    Ok(spdk_lib)
}

/// Compiles SPDK helper sources.
fn compile_spdk_helpers<P>(inc_dirs: P) -> Result<(), Error>
where
    P: IntoIterator,
    P::Item: AsRef<Path>,
{
    let files = vec![
        "helpers/logwrapper.h",
        "helpers/logwrapper.c",
        "helpers/nvme_helper.h",
        "helpers/nvme_helper.c",
        "helpers/spdk_helper.h",
        "helpers/spdk_helper.c",
    ];

    let mut src_files = Vec::new();

    for s in &files {
        match fs::canonicalize(&s) {
            Ok(p) => {
                println!("cargo:rerun-if-changed={}", p.to_str().unwrap());
                if p.extension().unwrap() == "c" {
                    src_files.push(p);
                }
            }
            Err(e) => {
                return Err(Error::Generic(format!(
                    "Bad SPDK helper source {}: {}",
                    s, e
                )))
            }
        }
    }

    cc::Build::new()
        .includes(inc_dirs)
        .files(src_files)
        .compile("helpers");

    Ok(())
}

fn main() {
    #![allow(unreachable_code)]
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    panic!("spdk-rs crate is only for x86_64 (Nehalem or later) and aarch64 (with crypto) ISAs.");

    #[cfg(not(target_os = "linux"))]
    panic!("spdk-rs crate works only on linux");

    // Configure SPDK library.
    println!("\nConfiguring SPDK library...");
    let spdk_lib = match configure_spdk() {
        Ok(c) => {
            println!("Successfully configured SPDK library");
            c
        }
        Err(e) => {
            eprintln!("\nFailed to configure SPDK: {}\n", e);
            std::process::exit(1);
        }
    };

    let inc_dirs = spdk_lib.get_inc_paths();

    // Compile SPDK helpers.
    println!("\nCompiling SPDK helpers...");
    match compile_spdk_helpers(&inc_dirs) {
        Ok(_) => {
            println!("Successfully compiled SPDK helpers");
        }
        Err(e) => {
            eprintln!("\nFailed to complie SPDK helpers: {}\n", e);
            std::process::exit(1);
        }
    }

    // Generate Rust bindings for SPDK.
    let clang_args: Vec<String> = inc_dirs
        .iter()
        .map(|p| format!("-I{}", p.to_str().unwrap()))
        .collect();

    let macros = Arc::new(RwLock::new(HashSet::new()));

    let bindings = bindgen::Builder::default()
        .clang_args(clang_args)
        .header("wrapper.h")
        .rustfmt_bindings(true)
        .allowlist_function("*.aio.*")
        .allowlist_function("*.crypto_disk.*")
        .allowlist_function("*.iscsi.*")
        .allowlist_function("*.lock_lba_range")
        .allowlist_function("*.lvol.*")
        .allowlist_function("*.lvs.*")
        .allowlist_function("*.uring.*")
        .allowlist_function("^iscsi.*")
        .allowlist_function("^spdk.*")
        .allowlist_function("create_malloc_disk")
        .allowlist_function("delete_malloc_disk")
        .allowlist_function("^bdev.*")
        .allowlist_function("^nbd_.*")
        .allowlist_function("^vbdev_.*")
        .allowlist_function("^nvme_cmd_.*")
        .allowlist_function("^nvme_status_.*")
        .allowlist_function("^nvmf_subsystem_find_listener")
        .allowlist_function("^nvmf_subsystem_set_ana_state")
        .allowlist_function("^nvmf_subsystem_set_cntlid_range")
        .allowlist_function("^nvmf_tgt_accept")
        .allowlist_function("^nvme_qpair_.*")
        .allowlist_function("^nvme_ctrlr_.*")
        .allowlist_function("^nvme_transport_qpair_.*")
        .blocklist_type("^longfunc")
        .allowlist_type("^spdk_nvme_ns_flags")
        .allowlist_type("^spdk_nvme_registered_ctrlr.*")
        .allowlist_type("^spdk_nvme_reservation.*")
        .allowlist_var("^NVMF.*")
        .allowlist_var("^SPDK.*")
        .allowlist_var("^spdk.*")
        .trust_clang_mangling(false)
        .opaque_type("^spdk_nvme_sgl_descriptor")
        .opaque_type("^spdk_nvme_ctrlr_data")
        .opaque_type("^spdk_nvme_feat_async_event_configuration.*")
        .opaque_type("^spdk_nvmf_fabric_connect.*")
        .opaque_type("^spdk_nvmf_fabric_prop.*")
        .layout_tests(false)
        .derive_default(true)
        .derive_debug(true)
        .derive_copy(true)
        .prepend_enum_name(false)
        .generate_inline_functions(true)
        .parse_callbacks(Box::new(MacroCallback {
            macros,
        }));

    #[cfg(target_arch = "x86_64")]
    let bindings = bindings.clang_arg("-march=nehalem");

    let bindings = bindings
        .generate()
        .expect("Unable to generate SPDK bindings");

    let out_path = get_out_dir();
    bindings
        .write_to_file(out_path.join("libspdk.rs"))
        .expect("Couldn't write SPDK bindings!");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
}
