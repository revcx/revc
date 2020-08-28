#![allow(warnings)]
#![allow(dead_code)]

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate lazy_static;

pub mod api;
mod dec;
mod enc;

mod def;
mod df;
mod ipred;
mod itdq;
mod mc;
mod picman;
mod plane;
mod recon;
mod region;
mod tbl;
mod tracer;
mod util;

mod hawktracer {
    cfg_if::cfg_if! {
      if #[cfg(feature="profile")] {
        pub use rust_hawktracer::*;
      } else {
        pub use noop_proc_macro::hawktracer;
      }
    }
}
