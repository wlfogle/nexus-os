//! smoltcp `phy::Device` implementation backed by the VirtIO-net driver.
//!
//! `NetDevice` is a zero-sized handle; all state lives in the global driver in
//! `drivers::virtio::net`.  Tokens copy frames to/from the heap so they never
//! hold the driver lock across smoltcp's packet processing.

use alloc::vec;
use alloc::vec::Vec;
use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use crate::drivers::virtio::net;

/// Largest ethernet frame (with header) we advertise to smoltcp.
const MTU: usize = 1514;

/// Zero-sized smoltcp device handle over the global VirtIO-net driver.
pub struct NetDevice;

impl Device for NetDevice {
    type RxToken<'a> = NetRxToken;
    type TxToken<'a> = NetTxToken;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let frame = net::receive_frame()?;
        Some((NetRxToken { frame }, NetTxToken))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        if net::present() {
            Some(NetTxToken)
        } else {
            None
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = MTU;
        caps
    }
}

/// Owns a received frame; hands it to smoltcp on `consume`.
pub struct NetRxToken {
    frame: Vec<u8>,
}

impl phy::RxToken for NetRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.frame)
    }
}

/// Builds a frame into a heap buffer, then transmits it via the driver.
pub struct NetTxToken;

impl phy::TxToken for NetTxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = vec![0u8; len];
        let result = f(&mut buf);
        net::transmit_frame(&buf);
        result
    }
}
