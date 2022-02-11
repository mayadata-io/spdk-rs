///! TODO
use std::{marker::PhantomData, os::raw::c_void, ptr::NonNull};

use crate::{
    ffihelper::{errno_error, ErrnoResult, IntoCString},
    libspdk::{
        spdk_bdev,
        spdk_bdev_close,
        spdk_bdev_desc,
        spdk_bdev_desc_get_bdev,
        spdk_bdev_event_type,
        spdk_bdev_open_ext,
        SPDK_BDEV_EVENT_MEDIA_MANAGEMENT,
        SPDK_BDEV_EVENT_REMOVE,
        SPDK_BDEV_EVENT_RESIZE,
    },
    Bdev,
    BdevOps,
};

/// Wrapper for `spdk_bdev_desc`.
/// TODO
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
#[derive(Debug)]
pub struct BdevDesc<BdevData>
where
    BdevData: BdevOps,
{
    /// TODO
    inner: NonNull<spdk_bdev_desc>,
    /// TODO
    _data: PhantomData<BdevData>,
}

// TODO: is `BdevDesc` really a Sync/Send type?
unsafe impl<T: BdevOps> Sync for BdevDesc<T> {}
unsafe impl<T: BdevOps> Send for BdevDesc<T> {}

impl<BdevData> BdevDesc<BdevData>
where
    BdevData: BdevOps,
{
    /// TODO
    ///
    /// # Arguments
    ///
    /// * `bdev_name`: TODO
    /// * `rw`: TODO
    /// * `event_cb`: TODO
    pub fn open(
        bdev_name: &str,
        rw: bool,
        event_cb: fn(BdevEvent, Bdev<BdevData>),
    ) -> ErrnoResult<Self> {
        let mut desc: *mut spdk_bdev_desc = std::ptr::null_mut();

        // let ctx = Box::new(BdevEventContext::<BdevData> {
        //     event_cb: Box::new(event_cb),
        // });

        let rc = unsafe {
            spdk_bdev_open_ext(
                bdev_name.into_cstring().as_ptr(),
                rw,
                Some(inner_bdev_event_cb::<BdevData>),
                // Box::into_raw(ctx) as *mut c_void,
                event_cb as *mut c_void,
                &mut desc,
            )
        };

        if rc != 0 {
            errno_error::<Self>(rc)
        } else {
            assert_eq!(desc.is_null(), false);
            Ok(Self::from_ptr(desc))
        }
    }

    /// TODO
    pub fn close(&mut self) {
        unsafe {
            // Close the desc.
            spdk_bdev_close(self.as_ptr());
        }
    }

    /// TODO
    /// Returns a Bdev associated with this descriptor.
    /// A descriptor cannot exist without a Bdev.
    pub fn bdev(&self) -> Bdev<BdevData> {
        let b = unsafe { spdk_bdev_desc_get_bdev(self.as_ptr()) };
        Bdev::from_inner_ptr(b)
    }

    /// Returns a pointer to the underlying `spdk_bdev_desc` structure.
    pub(crate) fn as_ptr(&self) -> *mut spdk_bdev_desc {
        self.inner.as_ptr()
    }

    /// TODO
    pub fn legacy_as_ptr(&self) -> *mut spdk_bdev_desc {
        self.as_ptr()
    }

    /// TODO
    ///
    /// # Arguments
    ///
    /// * `ptr`: TODO
    pub(crate) fn from_ptr(ptr: *mut spdk_bdev_desc) -> Self {
        Self {
            inner: NonNull::new(ptr).unwrap(),
            _data: Default::default(),
        }
    }

    /// TODO
    ///
    /// # Arguments
    ///
    /// * `ptr`: TODO
    pub fn legacy_from_ptr(ptr: *mut spdk_bdev_desc) -> Self {
        Self::from_ptr(ptr)
    }
}

impl<BdevData> Clone for BdevDesc<BdevData>
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

/// TODO
pub enum BdevEvent {
    /// TODO
    Remove,
    /// TODO
    Resize,
    /// TODO
    MediaManagement,
}

impl From<spdk_bdev_event_type> for BdevEvent {
    fn from(t: spdk_bdev_event_type) -> Self {
        match t {
            SPDK_BDEV_EVENT_REMOVE => BdevEvent::Remove,
            SPDK_BDEV_EVENT_RESIZE => BdevEvent::Resize,
            SPDK_BDEV_EVENT_MEDIA_MANAGEMENT => BdevEvent::MediaManagement,
            _ => panic!("Bad Bdev event type: {}", t),
        }
    }
}

impl From<BdevEvent> for spdk_bdev_event_type {
    fn from(t: BdevEvent) -> Self {
        match t {
            BdevEvent::Remove => SPDK_BDEV_EVENT_REMOVE,
            BdevEvent::Resize => SPDK_BDEV_EVENT_RESIZE,
            BdevEvent::MediaManagement => SPDK_BDEV_EVENT_MEDIA_MANAGEMENT,
        }
    }
}

/// TODO
///
/// # Generic Arguments
///
/// * `BdevData`: TODO
///
/// # Arguments
///
/// * `event`: TODO
/// * `bdev`: TODO
/// * `ctx`: TODO
unsafe extern "C" fn inner_bdev_event_cb<BdevData>(
    event: spdk_bdev_event_type,
    bdev: *mut spdk_bdev,
    ctx: *mut c_void,
) where
    BdevData: BdevOps,
{
    let ctx = std::mem::transmute::<_, fn(BdevEvent, Bdev<BdevData>)>(ctx);
    (ctx)(event.into(), Bdev::<BdevData>::from_inner_ptr(bdev));
}
