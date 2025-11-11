#![no_std]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

pub mod errors;
pub mod instructions;
pub mod states;

pinocchio_pubkey::declare_id!("8zASAJ7QL5t7S2oSTyAanSFFehAB2i3n4LiRHsZ6piuZ");