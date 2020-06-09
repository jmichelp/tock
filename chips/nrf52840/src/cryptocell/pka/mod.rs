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

use super::bitfields::*;
use kernel::common::registers::{register_structs, ReadOnly, ReadWrite, WriteOnly};

register_structs! {
    pub CryptoCellPkaRegisters {
        /// Memory mapped virtual register R0 to R31
        (0x0000 => pub memory_map: [ReadWrite<u32, MemoryMap::Register>; 32]),
        /// PKA's OPCODE
        (0x0080 => pub opcode: ReadWrite<u32, Opcode::Register>),
        /// N_NP_T0_T1
        (0x0084 => pub n_np_t0_t1: ReadWrite<u32, NNpT0T1::Register>),
        /// PKA pipe status
        (0x0088 => pub pka_status: ReadOnly<u32, PkaStatus::Register>),
        /// Software reset the PKA
        (0x008C => pub pka_sw_reset: WriteOnly<u32, Task::Register>),
        /// These registers hold the optional size of the operation in bytes
        (0x0090 => pub pka_lx: [ReadWrite<u32, OpSize::Register>; 8]),
        /// PKA pipe is ready to receive a new OPCODE
        (0x00B0 => pub pka_pipe_ready: ReadOnly<u32, Event::Register>),
        /// PKA operation is completed
        (0x00B4 => pub pka_done: ReadOnly<u32, Event::Register>),
        /// This register defines which PKA FSM monitor is being output
        (0x00B8 => pub pka_mon_select: ReadWrite<u32, MonitorSelect::Register>),
        (0x00BC => _reserved0),
        /// Version of the PKA
        (0x00C4 => pub pka_version: ReadOnly<u32>),
        (0x00C8 => _reserved1),
        /// PKA monitor bus register
        (0x00D0 => pub pka_mon_read: ReadOnly<u32>),
        /// First address given to PKA SRAM for write transactions
        (0x00D4 => pub pka_sram_addr: WriteOnly<u32>),
        /// Write data to PKA SRAM.
        /// Triggers the SRAM write DMA address to automatically be incremented
        (0x00D8 => pub pka_sram_wdata: WriteOnly<u32>),
        /// Read data from PKA SRAM
        /// Triggers the SRAM read DMA address to automatically be incremented
        (0x00DC => pub pka_sram_data: ReadOnly<u32>),
        /// Wirte buffer clean
        (0x00E0 => pub pka_sram_wr_clr: WriteOnly<u32>),
        /// First address given to PKA SRAM for read transactions
        (0x00E4 => pub pka_sram_raddr: WriteOnly<u32>),
        (0x00E8 => _reserved2),
        /// This register holds the data written to PKA memory using the wop opcode.
        (0x00F0 => pub pka_word_access: WriteOnly<u32>),
        (0x00F4 => _reserved3),
        /// This register maps the virtual buffer registers to a physical address in memory
        (0x00F8 => pub pka_buff_addr: WriteOnly<u32, PkaAddress::Register>),
        (0x00FC => @END),
    }
}
