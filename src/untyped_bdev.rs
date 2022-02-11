///! TODO
use crate::{Bdev, BdevIo, BdevOps, IoChannel, IoDevice, IoType};
use std::pin::Pin;

/// An alias for a "raw", untyped Bdev type.
/// TODO: better description.
pub type UntypedBdev = Bdev<()>;

/// Implementation of `BdevOps` for a dummy Bdev type.
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
    }

    fn io_type_supported(&self, _io_type: IoType) -> bool {
        false
    }

    fn get_io_device(&self) -> &Self::IoDev {
        &self
    }
}

//// Implementation of `IoDevice` for a dummy Bdev type.
impl IoDevice for () {
    type ChannelData = ();

    fn io_channel_create(self: Pin<&mut Self>) -> Self::ChannelData {
        ()
    }
}
