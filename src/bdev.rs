///! Wrappers for SPDK `spdk_bdev` structure and the related API.
use std::{
    ffi::CString,
    marker::PhantomData,
    marker::PhantomPinned,
    os::raw::c_void,
    pin::Pin,
    ptr::{null_mut, NonNull},
};

use nix::errno::Errno;

use crate::{
    ffihelper::{
        errno_result_from_i32,
        AsStr,
        ErrnoResult,
        FfiResult,
        IntoCString,
    },
    libspdk::{
        spdk_bdev,
        spdk_bdev_alias_add,
        spdk_bdev_alias_del,
        spdk_bdev_fn_table,
        spdk_bdev_get_aliases,
        spdk_bdev_get_buf_align,
        spdk_bdev_get_by_name,
        spdk_bdev_io_type_supported,
        spdk_bdev_module_release_bdev,
        spdk_bdev_register,
        spdk_bdev_unregister,
    },
    BdevIo,
    BdevModule,
    BdevOps,
    IoChannel,
    IoDevice,
    IoType,
    Uuid,
};

/// Wrapper for SPDK `spdk_bdev` structure and the related API.
/// This wrapper refers to a Bdev, it does not own it: Bdev lifecycle is managed
/// by SPDK. A single Bdev can be refer by multiple `Bdev<>` wrappers.
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
pub struct Bdev<BdevData>
where
    BdevData: BdevOps,
{
    /// TODO
    inner: NonNull<spdk_bdev>,
    /// TODO
    _data: PhantomData<BdevData>,
}

impl<BdevData> Bdev<BdevData>
where
    BdevData: BdevOps,
{
    /// Registers this Bdev in SPDK.
    /// TODO: comment
    /// TODO: Error / result
    pub fn register_bdev(&mut self) -> ErrnoResult<()> {
        let errno = unsafe { spdk_bdev_register(self.as_inner_ptr()) };
        errno_result_from_i32((), errno)
    }

    /// TODO
    pub fn unregister_bdev(&mut self) {
        unsafe {
            spdk_bdev_unregister(
                self.as_inner_ptr(),
                None,
                null_mut::<c_void>(),
            );
        }
    }

    /// Returns a Bdev module for this Bdev.
    pub fn module(&self) -> BdevModule {
        BdevModule::from_ptr(self.as_inner_ref().module)
    }

    /// Returns the name of the module for thos Bdev.
    pub fn module_name(&self) -> &str {
        unsafe { (*self.as_inner_ref().module).name.as_str() }
    }

    /// TODO
    /// ... lookup a bdev by its name
    pub fn lookup_by_name(name: &str) -> Option<Self> {
        let name = String::from(name).into_cstring();
        let bdev = unsafe { spdk_bdev_get_by_name(name.as_ptr()) };
        if bdev.is_null() {
            None
        } else {
            Some(Self::from_inner_ptr(bdev))
        }
    }

    /// Returns by a Bdev module who has claimed this Bdev.
    pub fn claimed_by_module(&self) -> Option<BdevModule> {
        let ptr = self.as_inner_ref().internal.claim_module;
        if ptr.is_null() {
            None
        } else {
            Some(BdevModule::from_ptr(ptr))
        }
    }

    /// Returns by a name of Bdev module who has claimed this Bdev.
    pub fn claimed_by(&self) -> Option<String> {
        self.claimed_by_module().map(|m| m.name().to_string())
    }

    /// Returns Bdev name.
    pub fn name(&self) -> &str {
        self.as_inner_ref().name.as_str()
    }
    /// Returns the configured product name.
    pub fn product_name(&self) -> &str {
        self.as_inner_ref().product_name.as_str()
    }

    /// Returns Bdev's raw (SPDK representation) UUID.
    pub fn raw_uuid(&self) -> Uuid {
        Uuid::new(&self.as_inner_ref().uuid)
    }

    /// Sets Bdev's raw (SPDK representation) UUID.
    pub unsafe fn set_raw_uuid(&mut self, uuid: Uuid) {
        self.as_inner_mut().uuid = uuid.into_raw();
    }

    /// Returns Bdev's UUID.
    pub fn uuid(&self) -> uuid::Uuid {
        self.raw_uuid().into()
    }

    /// Returns the UUID of this bdev as a hyphenated string.
    pub fn uuid_as_string(&self) -> String {
        self.uuid().to_hyphenated().to_string()
    }

    /// TODO
    /// Set a list of aliases on the bdev, used to find the bdev later
    pub fn add_aliases(&mut self, alias: &[String]) -> bool {
        alias
            .iter()
            .filter(|a| -> bool { !self.add_alias(a) })
            .count()
            == 0
    }

    /// TODO
    /// Set an alias on the bdev, this alias can be used to find the bdev later.
    /// If the alias is already present we return true
    pub fn add_alias(&mut self, alias: &str) -> bool {
        let alias = alias.into_cstring();
        let ret =
            unsafe { spdk_bdev_alias_add(self.as_inner_ptr(), alias.as_ptr()) }
                .to_result(Errno::from_i32);

        matches!(ret, Err(Errno::EEXIST) | Ok(_))
    }

    /// Removes the given alias from the Bdev.
    pub fn remove_alias(&mut self, alias: &str) {
        unsafe {
            spdk_bdev_alias_del(
                self.as_inner_ptr(),
                alias.into_cstring().as_ptr(),
            )
        };
    }

    /// Returns a list of Bdev aliases.
    pub fn aliases(&self) -> Vec<String> {
        let mut aliases = Vec::new();
        let head = unsafe { &*spdk_bdev_get_aliases(self.as_inner_ptr()) };
        let mut ent_ptr = head.tqh_first;
        while !ent_ptr.is_null() {
            let ent = unsafe { &*ent_ptr };
            let alias = ent.alias.name.as_str();
            aliases.push(alias.to_string());
            ent_ptr = ent.tailq.tqe_next;
        }
        aliases
    }

    /// Returns the block size of the underlying device.
    pub fn block_len(&self) -> u32 {
        self.as_inner_ref().blocklen
    }

    /// Sets the block size of the underlying device.
    pub unsafe fn set_block_len(&mut self, len: u32) {
        self.as_inner_mut().blocklen = len;
    }

    /// Returns number of blocks for this device.
    pub fn num_blocks(&self) -> u64 {
        self.as_inner_ref().blockcnt
    }

    /// Sets number of blocks for this device.
    pub unsafe fn set_num_blocks(&mut self, count: u64) {
        self.as_inner_mut().blockcnt = count
    }

    /// Returns the Bdev size in bytes.
    pub fn size_in_bytes(&self) -> u64 {
        self.num_blocks() * (self.block_len() as u64)
    }

    /// Returns the alignment of the Bdev.
    pub fn alignment(&self) -> u64 {
        unsafe { spdk_bdev_get_buf_align(self.as_inner_ptr()) }
    }

    /// Returns the required alignment of the Bdev.
    pub fn required_alignment(&self) -> u8 {
        self.as_inner_ref().required_alignment
    }

    /// Returns true if this Bdev is claimed by some other component.
    pub fn is_claimed(&self) -> bool {
        !self.as_inner_ref().internal.claim_module.is_null()
    }

    /// Returns true if this Bdev is claimed by the given Bdev module.
    pub fn is_claimed_by_module(&self, module: &BdevModule) -> bool {
        self.as_inner_ref().internal.claim_module == module.as_ptr()
    }

    /// Releases a write claim on a block device.
    pub fn release_claim(&self) {
        if self.is_claimed() {
            unsafe {
                spdk_bdev_module_release_bdev(self.as_inner_ptr());
            }
        }
    }

    /// Determines whenever the Bdev supports the requested I/O type.
    pub fn io_type_supported(&self, io_type: IoType) -> bool {
        unsafe {
            spdk_bdev_io_type_supported(self.as_inner_ptr(), io_type.into())
        }
    }

    /// Returns a reference to a data object associated with this Bdev.
    pub fn data<'a>(&self) -> Pin<&'a BdevData> {
        unsafe {
            let c = self.as_inner_ref().ctxt as *const Container<BdevData>;
            Pin::new_unchecked(&(*c).data)
        }
    }

    /// Returns a mutable reference to a data object associated with this Bdev.
    pub fn data_mut<'a>(&mut self) -> Pin<&'a mut BdevData> {
        unsafe {
            let c = self.as_inner_ref().ctxt as *mut Container<BdevData>;
            Pin::new_unchecked(&mut (*c).data)
        }
    }

    /// Returns a pointer to the underlying `spdk_bdev` structure.
    pub(crate) fn as_inner_ptr(&self) -> *mut spdk_bdev {
        self.inner.as_ptr()
    }

    /// Returns a reference to the underlying `spdk_bdev` structure.
    pub(crate) fn as_inner_ref(&self) -> &spdk_bdev {
        unsafe { self.inner.as_ref() }
    }

    /// Returns a mutable reference to the underlying `spdk_bdev` structure.
    pub(crate) fn as_inner_mut(&mut self) -> &mut spdk_bdev {
        unsafe { self.inner.as_mut() }
    }

    /// Creates a new `Bdev` wrapper from an SPDK structure pointer.
    pub(crate) fn from_inner_ptr(ptr: *mut spdk_bdev) -> Self {
        Self {
            inner: NonNull::new(ptr).unwrap(),
            _data: Default::default(),
        }
    }

    /// Public version of `as_inner_ptr()`.
    pub unsafe fn unsafe_inner_ptr(&self) -> *const spdk_bdev {
        self.as_inner_ptr()
    }

    /// Public version of `as_inner_ptr()`.
    pub unsafe fn unsafe_inner_mut_ptr(&mut self) -> *mut spdk_bdev {
        self.as_inner_ptr()
    }

    /// Public version of `from_inner_ptr()`.
    pub unsafe fn unsafe_from_inner_ptr(ptr: *mut spdk_bdev) -> Self {
        Self::from_inner_ptr(ptr)
    }
}

impl<BdevData> Clone for Bdev<BdevData>
where
    BdevData: BdevOps,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            _data: Default::default(),
        }
    }
}

/// Container for the data associated with a `Bdev` instance.
/// This container stores the `spdk_bdev` structure itself,
/// its associated function table and user-defined data structure provided upon
/// Bdev creation.
///
/// When SPDK destructs a BDEV, this container is dropped,
/// automatically freeing all the resources allocated during BDEV creation.
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
#[repr(C)]
pub(crate) struct Container<BdevData>
where
    BdevData: BdevOps,
{
    /// TODO
    pub(crate) bdev: spdk_bdev,
    /// TODO
    pub(crate) fn_table: spdk_bdev_fn_table,
    /// TODO
    pub(crate) data: BdevData,
    /// Prevent auto-Unpin.
    pub(crate) _pin: PhantomPinned,
}

impl<BdevData> Drop for Container<BdevData>
where
    BdevData: BdevOps,
{
    fn drop(&mut self) {
        // Tell the Bdev data object to be cleaned up.
        let pinned_data = unsafe { Pin::new_unchecked(&mut self.data) };
        pinned_data.destruct();

        // Drop the associated strings.
        unsafe {
            CString::from_raw(self.bdev.name);
            CString::from_raw(self.bdev.product_name);
        }
    }
}

impl<BdevData> Container<BdevData>
where
    BdevData: BdevOps,
{
    /// Creates a new container reference from an SPDK Bdev context
    /// pointer.
    ///
    /// # Safety
    ///
    /// TODO
    pub(crate) fn from_ptr<'a>(ctx: *const c_void) -> Pin<&'a Self> {
        unsafe { Pin::new_unchecked(&*(ctx as *const Self)) }
    }

    /// Creates a new mutable container reference from an SPDK Bdev context
    /// pointer.
    ///
    /// # Safety
    ///
    /// TODO
    pub(crate) fn from_ptr_mut<'a>(ctx: *mut c_void) -> Pin<&'a mut Self> {
        unsafe { Pin::new_unchecked(&mut *(ctx as *mut Self)) }
    }
}
