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
use crate::cryptocell::CryptoCell310;
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::hil;
use kernel::ReturnCode;

register_bitfields! [u32,
    AesControl [
        /// This field determines whether the AES performs Decrypt/Encrypt
        /// operations, in non-tunneling operations
        DEC_KEY0 OFFSET(0) NUMBITS(1) [
            Encrypt = 0,
            Decrypt = 1
        ],
        /// If MODE_KEY0 is set to 3'b001 (CBC), and this field is set - the
        /// mode is CBC-CTS. In addition, If MODE_KEY0 is set to 3'b010 (CTR),
        /// and this field is set - the mode is GCTR
        MODE0_IS_CBC_CTS OFFSET(1) NUMBITS(1) [],
        /// This field determines the AES mode in non tunneling operations, and
        /// the AES mode of the first stage in tunneling operations
        MODE_KEY0 OFFSET(2) NUMBITS(3) [
            ECB = 0,
            CBC = 1,
            CTR = 2,
            CBC_MAC = 3,
            XEX_XTS = 4,
            XCBC_MAC = 5,
            OFB = 6,
            CMAC = 7
        ],
        /// This field determines the AES mode of the second stage operation in
        /// tunneling operations
        MODE_KEY1 OFFSET(5) NUMBITS(3) [
            ECB = 0,
            CBC = 1,
            CTR = 2,
            CBC_MAC = 3,
            XEX_XTS = 4,
            XCBC_MAC = 5,
            OFB = 6,
            CMAC = 7
        ],
        /// If MODE_KEY0 is set to 3'b001 (CBC), and this field is set - the
        /// mode is CBC-with ESSIV
        CBC_IS_ESSIV OFFSET(8) NUMBITS(1) [],
        /// This field determines whether the AES performs dual-tunnel
        /// operations or standard non-tunneling operations
        AES_TUNNEL_IS_ON OFFSET(10) NUMBITS(1) [
            Standard = 0,
            Tunneling = 1
        ],
        /// If MODE_KEY0 is set to 3'b001 (CBC), and this field is set - the
        /// mode is BITLOCKER.
        CBC_IS_BITLOCKER OFFSET(11) NUMBITS(1) [],
        /// This field determines the AES Key length in non tunneling
        /// operations, and the AES key length of the first stage in
        /// tunneling operations
        NK_KEY0 OFFSET(12) NUMBITS(2) [
            Bits128 = 0,
            Bits192 = 1,
            Bits256 = 2,
            NA = 3
        ],
        /// This field determines the AES key length of the second stage
        /// operation in tunneling operations
        NK_KEY1 OFFSET(14) NUMBITS(2) [
            Bits128 = 0,
            Bits192 = 1,
            Bits256 = 2,
            NA = 3
        ],
        /// This field determines whether the second tunnel stage performs
        /// encrypt or decrypt operation
        AES_TUNNEL1_DECRYPT OFFSET(22) NUMBITS(1) [
            Encrypt = 0,
            Decrypt = 1
        ],
        /// This field determines, for tunneling operations, the data that is
        /// fed to the second tunneling stage
        AES_TUN_B1_USES_PADDED_DATA_IN OFFSET(23) NUMBITS(1) [
            FirstBlock = 0,
            DataIn = 1
        ],
        /// This field determines whether the first tunnel stage performs
        /// encrypt or decrypt operation
        AES_TUNNEL0_ENCRYPT OFFSET(24) NUMBITS(1) [
            Decrypt = 0,
            Encrypt = 1
        ],
        /// This fields determines whether the AES output is the result of the
        /// first or second tunneling stage
        AES_OUTPUT_MID_TUNNEL_DATA OFFSET(25) NUMBITS(1) [
            SecondTunnel = 0,
            FirstTunnel = 1
        ],
        /// This field determines whether the input data to the second tunnel
        /// stage is padded with zeroes (according to the remaining_bytes
        /// value) or not
        AES_TUNNEL_B1_PAD_EN OFFSET(26) NUMBITS(1) [
            NotPadded = 0,
            ZeroPadded = 1
        ],
        /// This field determines for AES-TO-HASH-AND-DOUT tunneling
        /// operations, whether the AES outputs to the HASH the result of the
        /// first or the second tunneling stage
        AES_OUT_MID_TUN_TO_HASH OFFSET(28) NUMBITS(1) [
            SecondTunnel = 0,
            FirstTunnel = 1
        ],
        /// This field determines the value that is written to AES_KEY0, when
        /// AES_SK is kicked
        AES_XOR_CRYPTOKEY OFFSET(29) NUMBITS(1) [
            HwCryptoKey = 0,
            HwXorKey0 = 1
        ],
        /// Using direct access and not the din-dout interface
        DIRECT_ACCESS OFFSET(31) NUMBITS(1) [
            DinDout = 0,
            Direct = 1
        ]
    ],

    AesHwFlags [
        SUPPORT_256_192_KEY OFFSET(0) NUMBITS(1),
        AES_LARGE_RKEK OFFSET(1) NUMBITS(1),
        DPA_CNTRMSR_EXIST OFFSET(2) NUMBITS(1),
        CTR_EXIST OFFSET(3) NUMBITS(1),
        ONLY_ENCRYPT OFFSET(4) NUMBITS(1),
        USE_SBOX_TABLE OFFSET(5) NUMBITS(1),
        USE_5_SBOXES OFFSET(8) NUMBITS(1),
        AES_SUPPORT_PREV_IV OFFSET(9) NUMBITS(1),
        AES_TUNNEL_EXIST OFFSET(10) NUMBITS(1),
        SECOND_REGS_SET_EXIST OFFSET(11) NUMBITS(1),
        DFA_CNTRMSR_EXIST OFFSET(12) NUMBITS(1)
    ]
];

register_structs! {
    pub CryptoCellAesRegisters {
        /// AES Key0 registers
        /// This key is used in non-tunneling operations, and as the first tunnel stage key
        /// in tunneling operations.
        (0x0000 => pub key0: [WriteOnly<u32>; 8]),
        /// AES Key1 registers
        /// This key is used as the second AES tunnel key in tunneling operations.
        (0x0020 => pub key1: [WriteOnly<u32>; 8]),
        /// AES IV0 is used as the AES IV register in non-tunneling operations and
        /// as the first tunnel stage IV register in tunneling operations.
        /// The IV register should be loaded accordingly to the AES mode.
        (0x0040 => pub iv0: [ReadWrite<u32>; 4]),
        /// AES IV1 is used as the AES IV register as the second tunnel stage IV register
        /// in tunneling operations.
        /// The IV register should be loaded according to the AES mode.
        (0x0050 => pub iv1: [ReadWrite<u32>; 4]),
        /// AES CTR0 is used as the the AES CTR register in non-tunneling operations and
        /// as the first tunnel stage CTR register in tunneling operations.
        /// The CTR register should be loaded according to the AES mode.
        (0x0060 => pub ctr0: [ReadWrite<u32>; 4]),
        /// This register is set when the AES core is active.
        (0x0070 => pub busy: ReadOnly<u32, Busy::Register>),
        (0x0074 => _reserved0),
        /// Writing to this register causes sampling of the HW key to the AES_KEY0 register.
        (0x0078 => pub sk: WriteOnly<u32, Task::Register>),
        /// Writing to this register trigers the AES engine generating of K1 and K2 for AES
        /// CMAC operations
        (0x007C => pub cmac_init: WriteOnly<u32, Task::Register>),
        (0x0080 => _reserved1),
        /// Writing to this address causes sampling of the HW key to the AES_KEY1 register
        (0x00B4 => pub sk1: WriteOnly<u32, Task::Register>),
        (0x00B8 => _reserved2),
        /// This register should be set with the amount of remaining bytes until the end of the current
        /// AES operation. The AES engine counts down from this value to determine the last / one before last
        /// blocks in AES CMAC, XTS AES and AES CCM
        (0x00BC => pub remaining_bytes: ReadWrite<u32>),
        /// This register holds the configuration of the AES engine
        (0x00C0 => pub control: ReadWrite<u32, AesControl::Register>),
        (0x00C4 => _reserved3),
        /// This register holds the pre-synthesis HW flag configuration of the AES engine
        (0x00C8 => pub hw_flags: ReadOnly<u32, AesHwFlags::Register>),
        (0x00CC => _reserved4),
        /// This register enables the AES CTR no increment mode
        (0x00D8 => pub ctr_no_increment: ReadWrite<u32, Task::Register>),
        (0x00DC => _reserved5),
        /// This register enables the AES DFA
        (0x00F0 => pub dfa_enable: ReadWrite<u32, Task::Register>),
        (0x00F4 => _reserved6),
        /// DFA error status register
        (0x00F8 => pub dfa_err_status: ReadOnly<u32, Bool::Register>),
        (0x00FC => _reserved7),
        /// Writing to this register triggers the AES engine to perform a CMAC operation with size 0.
        /// The CMAC result can be read from the AES_IV0 register
        (0x0124 => pub cmac_size0_kick: ReadWrite<u32, Task::Register>),
        (0x0128 => @END),
    }
}

impl<'a> hil::symmetric_encryption::AES128<'a> for CryptoCell310<'a> {
    fn enable(&self) {
        //self.aes.enable();
    }

    fn disable(&self) {
        //self.aes.disable();
    }

    fn set_client(&'a self, client: &'a dyn hil::symmetric_encryption::Client<'a>) {
        self.aes_client.set(client);
    }

    fn set_key(&self, key: &[u8]) -> ReturnCode {
        if key.len() != hil::symmetric_encryption::AES128_KEY_SIZE {
            return ReturnCode::EINVAL;
        }

        for i in 0..4 {
            let mut k = key[i * 4 + 0] as usize;
            k |= (key[i * 4 + 1] as usize) << 8;
            k |= (key[i * 4 + 2] as usize) << 16;
            k |= (key[i * 4 + 3] as usize) << 24;
            self.registers.aes.key0[i].set(k as u32);
        }

        ReturnCode::SUCCESS
    }

    fn set_iv(&self, iv: &[u8]) -> ReturnCode {
        if iv.len() != hil::symmetric_encryption::AES128_BLOCK_SIZE {
            return ReturnCode::EINVAL;
        }

        // Set the initial value from the array.
        for i in 0..4 {
            let mut c = iv[i * 4 + 0] as usize;
            c |= (iv[i * 4 + 1] as usize) << 8;
            c |= (iv[i * 4 + 2] as usize) << 16;
            c |= (iv[i * 4 + 3] as usize) << 24;
            // TODO(jmichel): depending on the mode we should assign the IV to a different register
            // But the current HIL doesn't guarantee that the API will be called in a given order.
            // So at the moment, let's always assign the IV to both registers.
            self.registers.aes.iv0[i].set(c as u32);
            // Normally the IV goes there only if the mode is CTR or OFB
            self.registers.aes.ctr0[i].set(c as u32);
        }

        ReturnCode::SUCCESS
    }

    fn start_message(&self) {
        if self.registers.aes.busy.is_set(Busy::BUSY) {
            return;
        }
        self.registers.ctrl.crypto_ctl.write(CryptoMode::MODE::Aes);
        while self.registers.aes.busy.is_set(Busy::BUSY) {}
        while self.registers.dout.mem_dma_busy.is_set(Busy::BUSY) {}
        while self.registers.din.mem_dma_busy.is_set(Busy::BUSY) {}
        while self.registers.dout.sram_dma_busy.is_set(Busy::BUSY) {}
        while self.registers.din.sram_dma_busy.is_set(Busy::BUSY) {}
        if self
            .registers
            .aes
            .control
            .matches_all(AesControl::MODE_KEY0::CMAC)
        {
            self.registers.aes.cmac_init.write(Task::ENABLE::SET);
        }
        self.registers.aes.remaining_bytes.set(0);
    }

    fn crypt(
        &'a self,
        source: Option<&'a mut [u8]>,
        dest: &'a mut [u8],
        start_index: usize,
        stop_index: usize,
    ) -> Option<(ReturnCode, Option<&'a mut [u8]>, &'a mut [u8])> {
        if self.registers.aes.busy.is_set(Busy::BUSY) {
            Some((ReturnCode::EBUSY, source, dest))
        } else {
            /*self.source.put(source);
            self.dest.replace(dest);
            if self.try_set_indices(start_index, stop_index) {
                self.dlli_write_block();
                None
            } else {
                Some((
                    ReturnCode::EINVAL,
                    self.source.take(),
                    self.dest.take().unwrap(),
                ))
            }*/
            None
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
enum ConfidentialityMode {
    ECB = 0,
    CBC = 1,
    CFB = 2,
    OFB = 3,
    CTR = 4,
}

impl<'a> hil::symmetric_encryption::AES128Ctr for CryptoCell310<'a> {
    fn set_mode_aes128ctr(&self, encrypting: bool) {
        //self.aes.set_mode(encrypting, ConfidentialityMode::CTR);
    }
}

impl<'a> hil::symmetric_encryption::AES128CBC for CryptoCell310<'a> {
    fn set_mode_aes128cbc(&self, encrypting: bool) {
        //self.aes.set_mode(encrypting, ConfidentialityMode::CBC);
    }
}
