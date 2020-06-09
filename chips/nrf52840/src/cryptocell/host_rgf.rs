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
    pub CryptoCellHostRgfRegisters {
        /// The Interrupt Request register.
        /// Each bit of this register holds the interrupt status of a single interrupt source.
        (0x0000 => pub interrupts: ReadOnly<u32, Interrupts::Register>),
        /// The Interrupt Mask register. Each bit of this register holds the mask of a single
        /// interrupt source.
        (0x0004 => pub interrupt_mask: ReadWrite<u32, Interrupts::Register>),
        /// Interrupt Clear Register
        (0x0008 => pub interrupt_clear: WriteOnly<u32, Interrupts::Register>),
        /// This register defines the endianness of the Host-accessible registers.
        (0x000C => pub endian: ReadWrite<u32, RgfEndianness::Register>),
        (0x0010 => _reserved0),
        /// This register holds the CryptoCell product signature.
        (0x0024 => pub signature: ReadOnly<u32>),
        /// This register holds the values of CryptoCell's pre-synthesis flags
        (0x0028 => pub boot: ReadOnly<u32, BootFlags::Register>),
        (0x002C => _reserved1),
        /// AES hardware key select
        (0x0038 => pub cryptokey_select: ReadWrite<u32, CryptoKey::Register>),
        (0x003C => _reserved2),
        /// This write-once register is the K_PRTL lock register. When this register is set,
        /// K_PRTL can not be used and a zeroed key will be used instead. The value of this
        /// register is saved in the CRYPTOCELL AO power domain.
        (0x004C => pub iot_kprtl_lock: WriteOnly<u32, Task::Register>),
        /// This register holds bits of K_DR. The value of this register is saved in the
        /// CRYPTOCELL AO power domain. Reading from this address returns the K_DR valid
        /// status indicating if K_DR is successfully retained.
        /// Only iot_kdr[0] can be read. A read value of 0x1 indicated that the K_DR key is
        /// successfully retained.
        /// iot_kdr[0] contains bits 31:0
        /// iot_kdr[1] contains bits 63:32
        /// iot_kdr[2] contains bits 95:64
        /// iot_kdr[3] contains bits 127:96
        (0x0050 => pub iot_kdr: [ReadWrite<u32>; 4]),
        /// Controls lifecycle state (LCS) for CRYPTOCELL subsystem
        (0x0060 => pub iot_lcs: ReadWrite<u32, IotLcs::Register>),
        (0x0064 => _reserved3),
        /// This register enables the core clk gating by masking/enabling the cc_idle_state
        /// output signal.
        (0x0078 => pub clock_gating_enable: ReadWrite<u32, Bool::Register>),
        /// This register holds the idle indication of CC
        (0x007C => pub cc_is_idle: ReadOnly<u32, CryptoCellIdle::Register>),
        /// This register start the power-down sequence.
        (0x0080 => pub powerdown: ReadWrite<u32, Task::Register>),
        /// These inputs are to be statically tied to 0 or 1 by the customers.
        /// When such an input is set, the matching engines inputs are tied to zero and its
        /// outputs are disconnected, so that the engine will be entirely removed by Synthesis
        (0x0084 => pub remove_ghash: ReadOnly<u32, Bool::Register>),
        /// These inputs are to be statically tied to 0 or 1 by the customers.
        /// When such an input is set, the matching engines inputs are tied to zero and its
        /// outputs are disconnected, so that the engine will be entirely removed by Synthesis
        (0x0088 => pub remove_chacha: ReadOnly<u32, Bool::Register>),
        (0x008C => @END),
    }
}
