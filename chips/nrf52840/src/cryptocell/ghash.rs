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
    pub CryptoCellGhashRegisters {
        /// GHASH Key0
        (0x0000 => pub subkey0: [WriteOnly<u32>; 4]),
        /// GHASH IV0
        (0x0010 => pub iv0: [ReadWrite<u32>; 4]),
        /// This register is set when the GHASH core is active.
        (0x0020 => pub busy: ReadOnly<u32, Busy::Register>),
        /// Writing to this address sets the GHASH engine to be ready to a new GHASH operation.
        (0x0024 => pub init: WriteOnly<u32, Task::Register>),
        (0x0028 => @END),
    }
}
