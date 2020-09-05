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

#[cfg(feature = "bench")]
pub mod bench {
    pub mod api {
        pub use crate::api::*;
    }
    pub mod df {
        pub use crate::df::*;
    }
    pub mod ipred {
        pub use crate::ipred::*;
    }
    pub mod itdq {
        pub use crate::itdq::*;
    }
    pub mod mc {
        pub use crate::mc::*;
    }
    pub mod frame {
        pub use crate::api::frame::*;
    }
    pub mod plane {
        pub use crate::plane::*;
    }
    pub mod recon {
        pub use crate::recon::*;
    }
    pub mod region {
        pub use crate::region::*;
    }
    pub mod util {
        pub use crate::util::*;
    }
    pub mod me {
        pub use crate::enc::me::*;
    }
    pub mod mode {
        pub use crate::enc::mode::*;
    }
    pub mod pinter {
        pub use crate::enc::pinter::*;
    }
    pub mod pintra {
        pub use crate::enc::pintra::*;
    }
    pub mod sad {
        pub use crate::enc::sad::*;
    }
    pub mod tq {
        pub use crate::enc::tq::*;
    }
}
