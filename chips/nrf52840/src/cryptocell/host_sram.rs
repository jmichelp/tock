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
    pub CryptoCellHostSramRegisters {
        /// Read / Write data to SRAM
        (0x0000 => pub data: ReadWrite<u32>),
        /// First address given to SRAM DMA for read/write transactions from SRAM]
        (0x0004 => pub addr: WriteOnly<u32, SramAddress::Register>),
        /// The SRAM content is ready for read in SRAM_DATA
        (0x0008 => pub ready: ReadOnly<u32, Bool::Register>),
        (0x000C => @END),
    }
}
