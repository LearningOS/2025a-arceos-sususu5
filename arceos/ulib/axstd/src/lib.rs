//! # The ArceOS Standard Library
//!
//! The [ArceOS] Standard Library is a mini-std library, with an interface similar
//! to rust [std], but calling the functions directly in ArceOS modules, instead
//! of using libc and system calls.
//!
//! These features are exactly the same as those in [axfeat], they are used to
//! provide users with the selection of features in axfeat, without import
//! [axfeat] additionally:
//!
//! ## Cargo Features
//!
//! - CPU
//!     - `smp`: Enable SMP (symmetric multiprocessing) support.
//!     - `fp_simd`: Enable floating point and SIMD support.
//! - Interrupts:
//!     - `irq`: Enable interrupt handling support.
//! - Memory
//!     - `alloc`: Enable dynamic memory allocation.
//!     - `alloc-tlsf`: Use the TLSF allocator.
//!     - `alloc-slab`: Use the slab allocator.
//!     - `alloc-buddy`: Use the buddy system allocator.
//!     - `paging`: Enable page table manipulation.
//!     - `tls`: Enable thread-local storage.
//! - Task management
//!     - `multitask`: Enable multi-threading support.
//!     - `sched_fifo`: Use the FIFO cooperative scheduler.
//!     - `sched_rr`: Use the Round-robin preemptive scheduler.
//!     - `sched_cfs`: Use the Completely Fair Scheduler (CFS) preemptive scheduler.
//! - Upperlayer stacks
//!     - `fs`: Enable file system support.
//!     - `myfs`: Allow users to define their custom filesystems to override the default.
//!     - `net`: Enable networking support.
//!     - `dns`: Enable DNS lookup support.
//!     - `display`: Enable graphics support.
//! - Device drivers
//!     - `bus-mmio`: Use device tree to probe all MMIO devices.
//!     - `bus-pci`: Use PCI bus to probe all PCI devices.
//!     - `driver-ramdisk`: Use the RAM disk to emulate the block device.
//!     - `driver-ixgbe`: Enable the Intel 82599 10Gbit NIC driver.
//!     - `driver-bcm2835-sdhci`: Enable the BCM2835 SDHCI driver (Raspberry Pi SD card).
//! - Logging
//!     - `log-level-off`: Disable all logging.
//!     - `log-level-error`, `log-level-warn`, `log-level-info`, `log-level-debug`,
//!       `log-level-trace`: Keep logging only at the specified level or higher.
//!
//! [ArceOS]: https://github.com/arceos-org/arceos

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

#[cfg(any(feature = "alloc", feature = "alt_alloc"))]
extern crate alloc;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::{boxed, format, string, vec};

#[doc(no_inline)]
pub use core::{arch, cell, cmp, hint, marker, mem, ops, ptr, slice, str};

#[macro_use]
mod macros;

#[cfg(feature = "alloc")]
mod hash;

#[cfg(feature = "alloc")]
pub mod collections {
    use core::hash::Hash;
    use hashbrown::HashMap as RawHashMap;
    use crate::hash::FnvBuildHasher;

    pub struct HashMap<K, V>(RawHashMap<K, V, FnvBuildHasher>);

    impl<K, V> HashMap<K, V> {
        #[inline]
        pub fn new() -> Self
        where
            K: Eq + Hash,
        {
            Self(RawHashMap::with_hasher(FnvBuildHasher::new()))
        }

        #[inline]
        pub fn with_capacity(capacity: usize) -> Self
        where
            K: Eq + Hash,
        {
            Self(RawHashMap::with_capacity_and_hasher(capacity, FnvBuildHasher::new()))
        }
    }

    // Implement Deref for HashMap, so that we can use all the methods of hashbrown::HashMap
    impl<K, V> core::ops::Deref for HashMap<K, V> {
        type Target = RawHashMap<K, V, FnvBuildHasher>;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<K, V> core::ops::DerefMut for HashMap<K, V> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

pub mod env;
pub mod io;
pub mod os;
pub mod process;
pub mod sync;
pub mod thread;
pub mod time;

#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "net")]
pub mod net;
