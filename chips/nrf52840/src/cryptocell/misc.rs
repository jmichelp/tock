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

use crate::cryptocell::bitfields::{ClockStatus, Task};
use kernel::common::registers::{register_structs, ReadOnly, ReadWrite};

register_structs! {
    pub CryptoCellMiscRegisters {
        (0x0000 => _reserved0),
        /// The AES clock enable register
        (0x0010 => pub aes_clk_enable: ReadWrite<u32, Task::Register>),
        (0x0014 => _reserved1),
        /// The HASH clock enable register
        (0x0018 => pub hash_clk_enable: ReadWrite<u32, Task::Register>),
        /// The PKA clock enable register
        (0x001C => pub pka_clk_enable: ReadWrite<u32, Task::Register>),
        /// The DMA clock enable register
        (0x0020 => pub dma_clk_enable: ReadWrite<u32, Task::Register>),
        /// the CryptoCell clocks' status register
        (0x0024 => pub clk_status: ReadOnly<u32, ClockStatus::Register>),
        (0x0028 => _reserved2),
        /// CHACHA/SALSA clock enable register
        (0x0058 => pub chacha_clk_enable: ReadWrite<u32, Task::Register>),
        (0x005C => @END),
    }
}
