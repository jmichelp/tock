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
use crate::cryptocell::{CryptoCell310, DigestAlgorithm, HashMode};
use core::cmp;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::debug;
use kernel::hil;
use kernel::ReturnCode;

register_bitfields![u32,
    // HASH register bitfields
    HashSelect [
        AES_MAC OFFSET(0) NUMBITS(1) [
            Hash = 0,
            AesMac = 1
        ],
        GHASH OFFSET(1) NUMBITS(1) [
            Hash = 0,
            GHash = 1
        ]
    ],

    HashVersion [
        FIXES OFFSET(0) NUMBITS(8),
        MINOR OFFSET(8) NUMBITS(4),
        MAJOR OFFSET(12) NUMBITS(4)
    ],

    pub HashControl [
        // bit 2 is reserved but to simplify the logic we include it in the bitfield.
        MODE OFFSET(0) NUMBITS(4) [
            MD5 = 0,
            SHA1 = 1,
            SHA256 = 2,
            SHA224 = 10
        ]
    ],

    pub PaddingConfig [
        /// Enable Padding generation. must be reset upon completion of padding.
        DO_PAD OFFSET(2) NUMBITS(1)
    ],

    HashParam [
        /// Indicates the number of concurrent words the hash is using to compute signature
        CONCURRENT_WORD OFFSET(0) NUMBITS(4) [
            CW1 = 1,
            CW2 = 2
        ],
        /// Indicate if Hi adders are present for each Hi value or 1 adder is shared for all Hi.
        CH OFFSET(4) NUMBITS(4) [
            OneHi = 0,
            AllHi = 1
        ],
        /// Determine the granularity of word size
        DATA_WORD OFFSET(8) NUMBITS(4) [
            Bits32 = 0,
            Bits64 = 1
        ],
        /// Indicate if SHA-512 is present in the design. By default SHA-1 and SHA-256 are present.
        SHA_512_EXISTS OFFSET(12) NUMBITS(1) [],
        /// Indicate if pad block is present in the design.
        PAD_EXISTS OFFSET(13) NUMBITS(1) [],
        /// Indicate if MD5 is present in HW
        MD5_EXISTS OFFSET(14) NUMBITS(1) [],
        /// Indicate if HMAC logic is present in the design
        HMAC_EXISTS OFFSET(15) NUMBITS(1) [],
        /// Indicate if SHA-256 is present in the design
        SHA_256_EXISTS OFFSET(16) NUMBITS(1) [],
        /// Indicate if COMPARE digest logic is present in the design
        HMAC_COMPARE_EXISTS OFFSET(17) NUMBITS(1) [],
        /// Indicate if HASH to dout is present in the design
        DUMP_HASH_TO_DOUT_EXISTS OFFSET(18) NUMBITS(1) []
    ],

    HashEndianness [
        /// The default value is little-endian. The data and generation of padding can
        /// be swapped to be big-endian.
        ENDIAN OFFSET(0) NUMBITS(1) [
            BigEndian = 0,
            LittleEndian = 1
        ]
    ]
];

register_structs! {
    pub CryptoCellHashRegisters {
        /// Write initial hash value or read final hash value
        (0x0000 => pub hash: [ReadWrite<u32>; 9]),
        (0x0024 => _reserved0),
        /// HW padding automatically activated by engine.
        /// For the special case of ZERO bytes data vector this register should not be used! instead use HASH_PAD_CFG
        (0x0044 => pub auto_hw_padding: WriteOnly<u32, Task::Register>),
        /// This register is always xored with the input to the hash engine.
        /// It should be '0' if xored is not required.
        (0x0048 => pub xor_din: ReadWrite<u32>),
        (0x004C => _reserved1),
        /// Indication to HASH that the following data is to be loaded into initial value registers in HASH(H0:H15) or IV to AES MAC
        (0x0054 => pub load_init_state: WriteOnly<u32, Bool::Register>),
        (0x0058 => _reserved2),
        /// Select the AES MAC module rather than the hash module
        (0x0064 => pub hash_select: WriteOnly<u32, HashSelect::Register>),
        (0x0068 => _reserved3),
        /// HASH VERSION register
        (0x0170 => pub version: ReadOnly<u32, HashVersion::Register>),
        (0x0174 => _reserved4),
        /// Selects which HASH mode to run
        (0x0180 => pub control: ReadWrite<u32, HashControl::Register>),
        /// This register enables the hash hw padding.
        (0x0184 => pub padding: ReadWrite<u32, Task::Register>),
        /// HASH_PAD_CFG Register.
        (0x0188 => pub pad_config: ReadWrite<u32, PaddingConfig::Register>),
        /// This register hold the length of current hash operation
        (0x018C => pub hash_len_lsb: ReadWrite<u32>),
        /// This register hold the length of current hash operation
        (0x0190 => pub hash_len_msb: ReadWrite<u32>),
        (0x0194 => _reserved5),
        /// HASH_PARAM Register.
        (0x019C => pub param: ReadOnly<u32, HashParam::Register>),
        (0x01A0 => _reserved6),
        /// HASH_AES_SW_RESET Register.
        (0x01A4 => pub aes_sw_reset: WriteOnly<u32, Task::Register>),
        /// This register hold the HASH_ENDIANESS configuration.
        (0x01A8 => pub little_endian: ReadWrite<u32, Bool::Register>),
        (0x01AC => @END),
    }
}

const MD5_INIT_VALUE: [u32; 4] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476];
const SHA1_INIT_VALUE: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
const SHA224_INIT_VALUE: [u32; 8] = [
    0xC1059ED8, 0x367CD507, 0x3070DD17, 0xF70E5939, 0xFFC00B31, 0x68581511, 0x64F98FA7, 0xBEFA4FA4,
];
const SHA256_INIT_VALUE: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

impl<'a> hil::digest::Md5 for CryptoCell310<'a> {
    fn set_mode_md5(&self) -> Result<(), ReturnCode> {
        debug!("[CC310] Set mode MD5");
        self.hash_algo.set(HashMode::Digest(DigestAlgorithm::Md5));
        self.hash_digest_size.set(MD5_INIT_VALUE.len() as u32);
        let mut initial_value = self.hash_ctx.get();
        initial_value[..MD5_INIT_VALUE.len()].copy_from_slice(&MD5_INIT_VALUE);
        self.hash_ctx.set(initial_value);
        self.hash_total_size.set(0);
        Ok(())
    }
}

impl<'a> hil::digest::Sha1 for CryptoCell310<'a> {
    fn set_mode_sha1(&self) -> Result<(), ReturnCode> {
        debug!("[CC310] Set mode SHA1");
        self.hash_algo.set(HashMode::Digest(DigestAlgorithm::Sha1));
        self.hash_digest_size.set(SHA1_INIT_VALUE.len() as u32);
        let mut initial_value = self.hash_ctx.get();
        initial_value[..SHA1_INIT_VALUE.len()].copy_from_slice(&SHA1_INIT_VALUE);
        self.hash_ctx.set(initial_value);
        self.hash_total_size.set(0);
        Ok(())
    }
}

impl<'a> hil::digest::Sha224 for CryptoCell310<'a> {
    fn set_mode_sha224(&self) -> Result<(), ReturnCode> {
        debug!("[CC310] Set mode SHA224");
        self.hash_algo
            .set(HashMode::Digest(DigestAlgorithm::Sha224));
        self.hash_digest_size.set(SHA224_INIT_VALUE.len() as u32);
        let mut initial_value = self.hash_ctx.get();
        initial_value[..SHA224_INIT_VALUE.len()].copy_from_slice(&SHA224_INIT_VALUE);
        self.hash_ctx.set(initial_value);
        self.hash_total_size.set(0);
        Ok(())
    }
}

impl<'a> hil::digest::Sha256 for CryptoCell310<'a> {
    fn set_mode_sha256(&self) -> Result<(), ReturnCode> {
        debug!("[CC310] Set mode SHA256");
        self.hash_algo
            .set(HashMode::Digest(DigestAlgorithm::Sha256));
        self.hash_digest_size.set(SHA256_INIT_VALUE.len() as u32);
        let mut initial_value = self.hash_ctx.get();
        initial_value[..SHA256_INIT_VALUE.len()].copy_from_slice(&SHA256_INIT_VALUE);
        self.hash_ctx.set(initial_value);
        self.hash_total_size.set(0);
        Ok(())
    }
}

impl<'a> hil::digest::HMACSha256 for CryptoCell310<'a> {
    fn set_mode_hmacsha256(&self, key: &[u8; 32]) -> Result<(), ReturnCode> {
        debug!("[CC310] Set mode HMAC-SHA256(key={:?})", key);
        self.hash_algo.set(HashMode::Hmac(DigestAlgorithm::Sha256));
        self.hash_digest_size.set(SHA256_INIT_VALUE.len() as u32);

        let mut initial_value = self.hash_ctx.get();
        initial_value[..SHA256_INIT_VALUE.len()].copy_from_slice(&SHA256_INIT_VALUE);
        self.hash_ctx.set(initial_value);
        // Process OPAD and save hashing context
        let mut pad = self.hash_data_queue.get();
        for i in 0..pad.len() {
            pad[i] = 0x5c ^ (if i < key.len() { key[i] } else { 0 });
        }
        self.cc_hash_update(&pad, false);
        self.hash_total_size.set(0);
        let mut opad_hash = self.hash_hmac_opad_ctx.get();
        opad_hash.copy_from_slice(&self.hash_ctx.get());
        self.hash_hmac_opad_ctx.set(opad_hash);

        // Process IPAD
        for i in 0..pad.len() {
            pad[i] = 0x36 ^ (if i < key.len() { key[i] } else { 0 });
        }
        self.hash_ctx.set(initial_value);
        self.cc_hash_update(&pad, false);
        self.hash_total_size.set(0);
        Ok(())
    }
}

impl<'a> hil::digest::Digest<'a, [u8; 32]> for CryptoCell310<'a> {
    fn set_client(&'a self, client: &'a dyn hil::digest::Client<'a, [u8; 32]>) {
        self.sha256_client.set(client);
    }

    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ReturnCode, &'static mut [u8])> {
        self.hash_data_buff.set(Some(data));

        // Merge queued data and new buffer
        let data_slice = data.take();
        let slice_len = data_slice.len();
        debug!("[CC310] SHA256.add_data([u8; {}])", slice_len);
        let mut block = self.hash_data_queue.get();
        let mut processed_size = self.hash_total_size.get();
        let cursor_in_block = (processed_size % (block.len() as u64)) as usize;
        let left_in_block = block.len() - cursor_in_block;

        processed_size += slice_len as u64;
        self.hash_total_size.set(processed_size);
        if slice_len < left_in_block {
            block[cursor_in_block..(cursor_in_block + slice_len)].copy_from_slice(data_slice);
        } else {
            // Process current block
            let (this_block, rest) = data_slice.split_at(left_in_block);
            block[cursor_in_block..].copy_from_slice(this_block);
            self.cc_hash_update(&block, false);
            let end_offset = rest.len() - (rest.len() % block.len());
            let (full_blocks, tail) = rest.split_at(end_offset);
            self.cc_hash_update(&full_blocks, false);
            block[..tail.len()].copy_from_slice(tail);
        }
        self.sha256_client.map(move |client| {
            client.add_data_done(Ok(()), data_slice);
        });
        Ok(slice_len)
    }

    fn run(
        &'a self,
        digest: &'static mut [u8; 32],
    ) -> Result<(), (ReturnCode, &'static mut [u8; 32])> {
        // Process remaining data
        debug!("[CC310] SHA256.run()");

        let digest_size = self.hash_digest_size.get() as usize;
        let mut block = self.hash_data_queue.get();
        let processed_size = self.hash_total_size.get();
        let cursor_in_block = (processed_size % (block.len() as u64)) as usize;
        self.cc_hash_update(&block[..cursor_in_block], true);

        let ctx = self.hash_ctx.get();
        for i in 0..digest_size {
            digest[(4 * i)..(4 * i + 4)].copy_from_slice(&ctx[i].to_be_bytes());
        }

        // If we were computing HMAC, the hash above is only for inner pad.
        // Now we need to finish processing the outer pad.
        match self.hash_algo.get() {
            HashMode::Hmac(_) => {
                // Reload context from opad
                self.hash_ctx.set(self.hash_hmac_opad_ctx.get());
                self.cc_hash_update(&digest[..digest_size], true);
                let ctx = self.hash_ctx.get();
                for i in 0..digest_size {
                    digest[(4 * i)..(4 * i + 4)].copy_from_slice(&ctx[i].to_be_bytes());
                }
            }
            _ => {}
        };
        // TODO(jmichel): remove this
        self.hash_digest.set(Some(digest));
        debug!("[CC310] Triggering callback");
        self.sha256_client.map(|client| {
            client.hash_done(Ok(()), self.hash_digest.take().unwrap());
        });
        Ok(())
    }

    fn clear_data(&self) {
        let mut block = self.hash_data_queue.get();
        block.iter_mut().for_each(|b| *b = 0);
        self.hash_data_queue.set(block);

        let mut ctx = self.hash_ctx.get();
        ctx.iter_mut().for_each(|b| *b = 0);
        self.hash_ctx.set(ctx);

        let mut opad = self.hash_hmac_opad_ctx.get();
        opad.iter_mut().for_each(|b| *b = 0);
        self.hash_hmac_opad_ctx.set(opad);
    }
}
