/**
 * $File: lib.rs $
 * $Date: 2021-10-17 20:22:21 $
 * $Revision: $
 * $Creator: Jen-Chieh Shen $
 * $Notice: See LICENSE.txt for modification and distribution information
 *                   Copyright © 2021 by Shen, Jen-Chieh $
 */
extern crate unicode_normalization;

mod constants;
mod search;

pub use search::{SearchBase, LineInfo, score};
