///! TODO
use std::{ffi::CString, os::raw::c_void, ptr::null_mut};

use crate::{
    bdev::Container,
    ffihelper::IntoCString,
    libspdk::{
        spdk_bdev,
        spdk_bdev_fn_table,
        spdk_bdev_io,
        spdk_bdev_io_type,
        spdk_get_io_channel,
        spdk_io_channel,
        spdk_json_write_ctx,
    },
    Bdev,
    BdevIo,
    BdevModule,
    BdevOps,
    IoChannel,
    IoDevice,
    IoType,
    JsonWriteContext,
    Uuid,
};

/// Builder for `Bdev` structure.
///
/// # Generic Arguments
///
/// * `'m`: Lifetime of the corresponding `BdevModule` instance.
/// * `BdevData`: Type for the Bdev data structure associated with a `Bdev`.
///
/// # Safety
///
/// TODO
pub struct BdevBuilder<'m, BdevData>
where
    BdevData: BdevOps<BdevData = BdevData>,
{
    name: Option<CString>,
    product_name: Option<CString>,
    blocklen: Option<u32>,
    blockcnt: Option<u64>,
    required_alignment: Option<u8>,
    uuid: Option<Uuid>,
    module: &'m BdevModule,
    fn_table: Option<spdk_bdev_fn_table>,
    data: Option<BdevData>,
}

impl<'m, BdevData> BdevBuilder<'m, BdevData>
where
    BdevData: BdevOps<BdevData = BdevData>,
{
    /// Creates a new `BdevBuilder` instance.
    ///
    /// # Arguments
    ///
    /// * `bdev_mod`: TODO
    pub(crate) fn new(bdev_mod: &'m BdevModule) -> BdevBuilder<'m, BdevData> {
        BdevBuilder {
            name: None,
            product_name: None,
            required_alignment: None,
            blocklen: None,
            blockcnt: None,
            uuid: None,
            module: bdev_mod,
            fn_table: None,
            data: None,
        }
    }

    /// Sets the Bdev data object for the Bdev being created.
    /// This Bdev parameter is manadory.
    /// Creates a new `BdevBuilder` instance.
    ///
    /// # Arguments
    ///
    /// * `ctx`: TODO
    pub fn with_data(mut self, ctx: BdevData) -> Self {
        self.fn_table = Some(spdk_bdev_fn_table {
            destruct: Some(inner_bdev_destruct::<BdevData>),
            submit_request: Some(inner_bdev_submit_request::<BdevData>),
            io_type_supported: Some(inner_bdev_io_type_supported::<BdevData>),
            get_io_channel: Some(inner_bdev_get_io_channel::<BdevData>),
            dump_info_json: Some(inner_dump_info_json::<BdevData>),
            write_config_json: None,
            get_spin_time: None,
            get_module_ctx: Some(inner_bdev_get_module_ctx::<BdevData>),
            get_memory_domains: None,
        });
        self.data = Some(ctx);
        self
    }

    /// Sets a UUID for the Bdev being created.
    ///
    /// # Arguments
    ///
    /// * `u`: TODO
    pub fn with_uuid(mut self, u: Uuid) -> Self {
        self.uuid = Some(u);
        self
    }

    /// Sets a name for the Bdev being created.
    /// This Bdev parameter is manadory.
    ///
    /// # Arguments
    ///
    /// * `name`: TODO
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(String::from(name).into_cstring());
        self
    }

    /// Sets a product name for the Bdev being created.
    /// This Bdev parameter is manadory.
    ///
    /// # Arguments
    ///
    /// * `prod_name`: TODO
    pub fn with_product_name(mut self, prod_name: &str) -> Self {
        self.product_name = Some(String::from(prod_name).into_cstring());
        self
    }

    /// Sets Bdev block length.
    /// This Bdev parameter is manadory.
    ///
    /// # Arguments
    ///
    /// * `val`: TODO
    pub fn with_block_length(mut self, val: u32) -> Self {
        self.blocklen = Some(val);
        self
    }

    /// Sets Bdev block count.
    /// This Bdev parameter is manadory.
    ///
    /// # Arguments
    ///
    /// * `val`: TODO
    pub fn with_block_count(mut self, val: u64) -> Self {
        self.blockcnt = Some(val);
        self
    }

    /// Sets Bdev block required alignment.
    /// This Bdev parameter is manadory.
    ///
    /// # Arguments
    ///
    /// * `val`: TODO
    pub fn with_required_alignment(mut self, val: u8) -> Self {
        self.required_alignment = Some(val);
        self
    }

    /// Consumes a `BdevBuilder` instance and produces a new `Bdev` instance.
    pub fn build(self) -> Bdev<BdevData> {
        // Create a new container for the Bdev data, `spdk_bdev` itself and
        // the associated function table.
        // The context (pointer to the Container<> itself in our case) and
        // the function table are filled later, after cont is alloced.
        let cont = Box::new(Container {
            bdev: spdk_bdev {
                ctxt: null_mut::<c_void>(),
                name: self.name.expect("Bdev name must be set").into_raw(),
                aliases: Default::default(),
                product_name: self
                    .product_name
                    .expect("Bdev product name must be set")
                    .into_raw(),
                write_cache: Default::default(),
                blocklen: self.blocklen.expect("Bdeb block length must be set"),
                phys_blocklen: Default::default(),
                blockcnt: self.blockcnt.expect("Bdeb block count must be set"),
                write_unit_size: Default::default(),
                acwu: Default::default(),
                required_alignment: self
                    .required_alignment
                    .expect("Bdev required alignment must be set"),
                split_on_optimal_io_boundary: Default::default(),
                optimal_io_boundary: Default::default(),
                max_segment_size: Default::default(),
                max_num_segments: Default::default(),
                max_unmap: Default::default(),
                max_unmap_segments: Default::default(),
                max_write_zeroes: Default::default(),
                uuid: self.uuid.unwrap_or_else(Uuid::generate).into_raw(),
                md_len: Default::default(),
                md_interleave: Default::default(),
                dif_type: Default::default(),
                dif_is_head_of_md: Default::default(),
                dif_check_flags: Default::default(),
                zoned: Default::default(),
                zone_size: Default::default(),
                max_zone_append_size: Default::default(),
                max_open_zones: Default::default(),
                max_active_zones: Default::default(),
                optimal_open_zones: Default::default(),
                media_events: Default::default(),
                module: self.module.as_ptr(),
                fn_table: null_mut::<spdk_bdev_fn_table>(),
                internal: Default::default(),
            },
            fn_table: self.fn_table.expect("Bdev function table must be set"),
            data: self.data.expect("Bdev data must be set"),
            _pin: Default::default(),
        });

        // Consume the container and store a pointer to it within the
        // `spdk_bdev` context field. It will be converted back into
        // Box<> and dropped later upon Bdev destruction.
        let pcont = Box::into_raw(cont);

        // Fill the context field (our Container<>) and the function table,
        // and construct a `Bdev` wrapper.
        unsafe {
            (*pcont).bdev.fn_table = &(*pcont).fn_table;
            (*pcont).bdev.ctxt = pcont as *mut c_void;
            Bdev::from_inner_ptr(&mut (*pcont).bdev)
        }
    }
}

/// Called by SPDK when a Bdev is being destroyed.
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
///
/// # Arguments
///
/// * `ctx`: Pointer to a Bdev context, which is a pointer to `Container<_>` in
///   our case.
///
/// # Safety
///
/// TODO
unsafe extern "C" fn inner_bdev_destruct<BdevData>(ctx: *mut c_void) -> i32
where
    BdevData: BdevOps,
{
    // Dropping the container will drop all the associated resources:
    // the context, names, function table and `spdk_bdev` itself.
    Box::from_raw(ctx as *mut Container<BdevData>);
    0
}

/// TODO
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
///
/// # Arguments
///
/// * `chan`: TODO
/// * `bio`: TODO
///
/// # Safety
///
/// TODO
unsafe extern "C" fn inner_bdev_submit_request<BdevData>(
    chan: *mut spdk_io_channel,
    bio: *mut spdk_bdev_io,
) where
    BdevData: BdevOps<BdevData = BdevData>,
{
    let c = IoChannel::<BdevData::ChannelData>::from_ptr(chan);
    let b = BdevIo::<BdevData::BdevData>::from_ptr(bio);
    b.bdev().data().submit_request(c, b);
}

/// Called by SPDK when it needs a new I/O channel for the given Bdev.
/// This function forwards the call to SPDK `spdk_get_io_channel`, which in turn
/// allocates a channel for a I/O device associated with this Bdev.
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
///
/// # Arguments
///
/// * `ctx`: Pointer to a Bdev context, which is a pointer to `Container<_>` in
///   our case.
///
/// # Safety
///
/// TODO
unsafe extern "C" fn inner_bdev_get_io_channel<BdevData>(
    ctx: *mut c_void,
) -> *mut spdk_io_channel
where
    BdevData: BdevOps<BdevData = BdevData>,
{
    let c = Container::<BdevData>::from_ptr(ctx);
    let io_dev = c.data.get_io_device();
    spdk_get_io_channel(io_dev.get_io_device_id())
}

/// TODO
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
///
/// # Arguments
///
/// * `ctx`: Pointer to a Bdev context, which is a pointer to `Container<_>` in
///   our case.
///
/// # Safety
///
/// TODO
unsafe extern "C" fn inner_bdev_get_module_ctx<BdevData>(
    _ctx: *mut c_void,
) -> *mut c_void
where
    BdevData: BdevOps<BdevData = BdevData>,
{
    todo!()
}

/// Called by SPDK to determine if a particular I/O channel for the given Bdev.
/// This function forwards the call to SPDK `spdk_get_io_channel`, which in turn
/// allocates a channel for a I/O device associated with this Bdev.
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
///
/// # Arguments
///
/// * `ctx`: Pointer to a Bdev context, which is a pointer to `Container<_>` in
///   our case.
/// * `io_type`: TODO
///
/// # Safety
///
/// TODO
unsafe extern "C" fn inner_bdev_io_type_supported<BdevData>(
    ctx: *mut c_void,
    io_type: spdk_bdev_io_type,
) -> bool
where
    BdevData: BdevOps<BdevData = BdevData>,
{
    let c = Container::<BdevData>::from_ptr(ctx);
    c.data.io_type_supported(IoType::from(io_type))
}

/// TODO
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
///
/// # Arguments
///
/// * `ctx`: Pointer to a Bdev context, which is a pointer to `Container<_>` in
///   our case.
/// * `w`: TODO
///
/// # Safety
///
/// TODO
unsafe extern "C" fn inner_dump_info_json<BdevData>(
    ctx: *mut c_void,
    w: *mut spdk_json_write_ctx,
) -> i32
where
    BdevData: BdevOps<BdevData = BdevData>,
{
    let c = Container::<BdevData>::from_ptr(ctx);
    c.data.dump_info_json(JsonWriteContext::from_ptr(w));

    // TODO: error processing?
    0
}
