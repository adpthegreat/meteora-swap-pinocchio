mod generated;

use generated::*;

pub mod instructions {
    pub use super::generated::instructions::*;
}

pub mod errors {
    pub use super::generated::errors::*;
}

pub mod programs {
    pub use super::generated::programs::*;
}
