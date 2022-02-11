///! Definition of untyped Bdev alias and related types.
use crate::{Bdev, BdevIo, BdevOps, IoChannel, IoDevice, IoType};
use std::pin::Pin;

/// An alias for a Bdev whose type is unknown or not important.
pub type UntypedBdev = Bdev<()>;

/// Dummy implementation of `BdevOps` for an untyped Bdev.
/// This implementation is provided only to satisfy generics restrictions.
impl BdevOps for () {
    type ChannelData = ();
    type BdevData = ();
    type IoDev = ();

    fn destruct(self: Pin<&mut Self>) {}

    fn submit_request(
        &self,
        _chan: IoChannel<Self::ChannelData>,
        _bio: BdevIo<Self::BdevData>,
    ) {
        unreachable!()
    }

    fn io_type_supported(&self, _io_type: IoType) -> bool {
        unreachable!()
    }

    fn get_io_device(&self) -> &Self::IoDev {
        unreachable!()
    }
}

//// Dummy implementation of `IoDevice` for an untyped Bdev.
/// This implementation is provided only to satisfy generics restrictions.
impl IoDevice for () {
    type ChannelData = ();

    fn io_channel_create(self: Pin<&mut Self>) -> Self::ChannelData {
        unreachable!()
    }
}
