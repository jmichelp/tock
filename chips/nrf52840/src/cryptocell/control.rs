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

use crate::cryptocell::bitfields::*;
use kernel::common::registers::{register_structs, ReadOnly, ReadWrite, WriteOnly};

register_structs! {
    pub CryptoCellControlRegisters {
        /// Defines the cryptographic flow
        (0x0000 => pub crypto_ctl: WriteOnly<u32, CryptoMode::Register>),
        (0x0004 => _reserved0),
        /// This register is set whent the cryptographic core is busy
        (0x0010 => pub crypto_busy: ReadOnly<u32, Busy::Register>),
        (0x0014 => _reserved1),
        /// This register is set when the Hash engine is busy
        (0x001C => pub hash_busy: ReadOnly<u32, Busy::Register>),
        (0x0020 => _reserved2),
        /// Undocumented register used to check some sort of signature.
        (0x0028 => pub undocumented: ReadOnly<u32>),
        (0x002C => _reserved3),
        /// A general R/W register for firmware use
        (0x0030 => pub context_id: ReadWrite<u32, Byte::Register>),
        (0x0034 => @END),
    }
}
