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

use crate::cryptocell::bitfields::{Bool, Busy, CpuDataSize, Endianness, LliWord1, Task};
use kernel::common::registers::{register_structs, ReadOnly, ReadWrite, WriteOnly};

register_structs! {
    pub CryptoCellDinRegisters {
        /// This address can be used by the CPU to write data directly to the DIN buffer
        /// to be sent to engines
        (0x0000 => pub buffer: WriteOnly<u32>),
        (0x0004 => _reserved0),
        /// Indicates whether memoty (AXI) source DMA (DIN) is busy
        (0x0020 => pub mem_dma_busy: ReadOnly<u32, Busy::Register>),
        (0x0024 => _reserved1),
        /// This register is used in direct LLI mode - holds the location of the data source
        /// in the memory (AXI)
        (0x0028 => pub src_lli_word0: WriteOnly<u32>),
        /// This register is used in direct LLI mode - holds the number of bytes to be read
        /// from the memory (AXI).
        /// Writing to this register triggers the DMA.
        (0x002C => pub src_lli_word1: WriteOnly<u32, LliWord1::Register>),
        /// Location of data (start address) to be read from SRAM
        (0x0030 => pub sram_src_addr: ReadWrite<u32>),
        /// This register holds the size of the data (in bytes) to be read from the SRAM
        (0x0034 => pub sram_bytes_len: ReadWrite<u32>),
        /// This register holds the status of the SRAM DMA DIN
        (0x0038 => pub sram_dma_busy: ReadOnly<u32, Busy::Register>),
        /// This register defines the endianness fo the DIN interface to SRAM
        (0x003C => pub sram_endianness: ReadWrite<u32, Endianness::Register>),
        (0x0040 => _reserved2),
        /// This register holds the number of bytes to be transmitted using external DMA
        (0x0048 => pub cpu_data_size: WriteOnly<u32, CpuDataSize::Register>),
        (0x004C => _reserved3),
        /// DIN FIFO empty indication
        (0x0050 => pub fifo_empty: ReadOnly<u32, Bool::Register>),
        (0x0054 => _reserved4),
        /// Writing to this register reset the DIN_FIFO pointers
        (0x0058 => pub reset_pointer: WriteOnly<u32, Task::Register>),
        (0x005C => @END),
    },

    pub CryptoCellDoutRegisters {
        /// Cryptographic result - CPU can directly access it.
        (0x0000 => pub buffer: ReadOnly<u32>),
        (0x0004 => _reserved0),
        /// DOUT memory DMA busy - Indicates that memory (AXI) destination DMA (DOUT) is busy.
        (0x0020 => pub mem_dma_busy: ReadOnly<u32, Busy::Register>),
        (0x0024 => _reserved1),
        /// This register is used in direct LLI mode - holds the location of the data
        /// destination in the memory (AXI)
        (0x0028 => pub dst_lli_word0: WriteOnly<u32>),
        /// This register is used in direct LLI mode - holds the number of bytes to be
        /// written to the memory (AXI).
        (0x002C => pub dst_lli_word1: ReadWrite<u32, LliWord1::Register>),
        /// Location of result to be sent to in SRAM
        (0x0030 => pub sram_dest_addr: ReadWrite<u32>),
        /// This register holds the size of the data (in bytes) to be written to the SRAM.
        (0x0034 => pub sram_bytes_len: ReadWrite<u32>),
        /// This register holds the status of the SRAM DMA DOUT.
        (0x0038 => pub sram_dma_busy: ReadOnly<u32, Busy::Register>),
        /// This register defines the endianness of the DOUT interface from SRAM.
        (0x003C => pub sram_endianness: ReadWrite<u32, Endianness::Register>),
        (0x0040 => _reserved2),
        /// Indication that the next read from the CPU is the last one. This is needed
        /// only when the data size is NOT modulo 4 (e.g. HASH padding).
        (0x0044 => pub read_align_last: WriteOnly<u32, Bool::Register>),
        (0x0048 => _reserved3),
        /// DOUT_FIFO_EMPTY Register
        (0x0050 => pub fifo_empty: ReadOnly<u32, Bool::Register>),
        (0x0054 => @END),
    }
}
