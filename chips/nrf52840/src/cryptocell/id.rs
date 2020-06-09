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
use kernel::common::registers::{register_structs, ReadOnly};

register_structs! {
    pub CryptoCellIdRegisters {
        (0x0000 => _reserved0),
        (0x00C0 => pub peripheral_id_4: ReadOnly<u32, PeripheralId4::Register>),
        (0x00C4 => _pid_reserved: [ReadOnly<u32>; 3]),
        (0x00D0 => pub peripheral_id_0: ReadOnly<u32, PeripheralId0::Register>),
        (0x00D4 => pub peripheral_id_1: ReadOnly<u32, PeripheralId1::Register>),
        (0x00D8 => pub peripheral_id_2: ReadOnly<u32, PeripheralId2::Register>),
        (0x00DC => pub peripheral_id_3: ReadOnly<u32, PeripheralId3::Register>),
        (0x00E0 => pub component_id_0: ReadOnly<u32, ComponentId0::Register>),
        (0x00E4 => pub component_id_1: ReadOnly<u32, ComponentId1::Register>),
        (0x00E8 => pub component_id_2: ReadOnly<u32, ComponentId2::Register>),
        (0x00EC => pub component_id_3: ReadOnly<u32, ComponentId3::Register>),
        (0x00F0 => @END),
    }
}
