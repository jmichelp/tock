// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::cryptocell::bitfields::{
    Busy, ChachaByteOrder, ChachaControl, ChachaDebug, ChachaFlags, Task,
};
use kernel::common::registers::{register_structs, ReadOnly, ReadWrite, WriteOnly};

register_structs! {
    pub CryptoCellChachaRegisters {
        /// CHACHA general configuration
        (0x0000 => control: ReadWrite<u32, ChachaControl::Register>),
        /// CHACHA version
        (0x0004 => version: ReadOnly<u32>),
        /// CHACHA key
        (0x0008 => key: [WriteOnly<u32>; 8]),
        /// CHACHA IV
        (0x0028 => iv: [ReadWrite<u32>; 2]),
        /// This register is set when the CHACHA/SALSA core is active
        (0x0030 => busy: ReadOnly<u32, Busy::Register>),
        /// This register holds the pre-synthesis HW flag configuration of the CHACHA/SALSA engine
        (0x0034 => flags: ReadOnly<u32, ChachaFlags::Register>),
        /// The two first words (n) in the last row of the cipher matrix are the block counter.
        /// At the end of each block (512b), the block_cnt for the next block is written by HW
        /// to the block_cnt_lsb and block_cnt_msb registers. Need reset block counter,
        /// if start new message.
        (0x0038 => block_cnt_lsb: ReadWrite<u32>),
        (0x003C => block_cnt_msb: ReadWrite<u32>),
        /// Resets the CHACHA/SALSA engine
        (0x0040 => reset: WriteOnly<u32, Task::Register>),
        /// CHACHA_FOR_POLY_KEY
        (0x0044 => chacha_for_poly_key: [ReadOnly<u32>; 8]),
        /// CHACHA/SALSA DATA ORDER configuration.
        (0x0064 => byte_word_order: ReadWrite<u32, ChachaByteOrder::Register>),
        /// This register is used to debug the CHACHA engine
        (0x0068 => debug: ReadOnly<u32, ChachaDebug::Register>),
        (0x006C => @END),
    }
}
