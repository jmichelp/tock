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

use crate::cryptocell::bitfields::{AhbHnonsec, AhbProt, Bool};
use kernel::common::registers::{register_structs, ReadWrite};

register_structs! {
    pub CryptoCellAhbRegisters {
        /// This register forces the ahb transactions to be always singles.
        /// - Address: 0x0000 - 0x0004
        (0x0000 => pub force_singles: ReadWrite<u32, Bool::Register>),
        /// AHB PROT value
        /// - Address: 0x0004 - 0x0008
        (0x0004 => pub prot_value: ReadWrite<u32, AhbProt::Register>),
        /// This register holds AHB HMASTLOCK value
        /// - Address: 0x0008 - 0x000C
        (0x0008 => pub hmastlock: ReadWrite<u32, Bool::Register>),
        /// This register holds AHB HNONSEC value
        /// - Address: 0x000C - 0x0010
        (0x000C => pub hnonsec: ReadWrite<u32, AhbHnonsec::Register>),
        (0x0010 => @END),
    }
}
