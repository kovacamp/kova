//! Kova Scanner -- Solana token survival probability program.

use anchor_lang::prelude::*;

pub mod contexts;
pub mod errors;
pub mod state;
pub mod utils;

declare_id!("KovA5cAnNeR7xQwK8rY2NjmEv3bUnDL4sHfT9pRs1Wz");

#[program]
pub mod kova_scanner {
    use super::*;
}
