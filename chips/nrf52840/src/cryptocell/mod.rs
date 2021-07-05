// Copyright 2019 Google LLC
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

//! CryptoCell 310
//!
//! Author
//! -------------------
//!
//! * Author: Jean-Michel Picod <jmichel@google.com>
//! * Date: October 1 2019

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::common::registers::{register_structs, InMemoryRegister, ReadWrite};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil;
use kernel::hil::time;

mod aes;
mod ahb;
mod bitfields;
mod chacha;
mod control;
mod din_dout;
mod ghash;
mod hash;
mod host_rgf;
mod host_sram;
mod id;
mod misc;
mod pka;
mod trng;

register_structs! {
    NordicCC310Registers {
        (0x0000 => enable: ReadWrite<u32, bitfields::Task::Register>),
        (0x0004 => @END),
    },

    pub CryptoCellRegisters {
        /// PKA registers
        /// - Base address: 0x0000
        (0x0000 => pka: pka::CryptoCellPkaRegisters),
        (0x00FC => _reserved0),
        /// RNG registers
        /// - Base address: 0x0100
        (0x0100 => rng: trng::CryptoCellRngRegisters),
        (0x01E0 => _reserved1),
        /// Chacha registers
        /// - Base address: 0x0380
        (0x0380 => chacha: chacha::CryptoCellChachaRegisters),
        (0x03EC => _reserved2),
        /// AES registers
        /// - Base address: 0x0400
        (0x0400 => aes: aes::CryptoCellAesRegisters),
        (0x0528 => _reserved3),
        /// HASH registers
        /// - Base address: 0x0640
        (0x0640 => hash: hash::CryptoCellHashRegisters),
        (0x07EC => _reserved4),
        /// Misc registers
        /// - Base address: 0x0800
        (0x0800 => misc: misc::CryptoCellMiscRegisters),
        (0x085C => _reserved5),
        /// CryptoCell control registers
        /// - Base address: 0x0900
        (0x0900 => ctrl: control::CryptoCellControlRegisters),
        (0x0934 => _reserved6),
        /// GHASH registers
        /// - Base address: 0x0960
        (0x0960 => ghash: ghash::CryptoCellGhashRegisters),
        (0x0988 => _reserved7),
        /// HOST_RGF registers
        /// - Base address: 0x0A00
        (0x0A00 => host_rgf: host_rgf::CryptoCellHostRgfRegisters),
        (0x0A8C => _reserved8),
        /// AHB registers
        /// - Base address: 0x0B00
        (0x0B00 => ahb: ahb::CryptoCellAhbRegisters),
        (0x0B10 => _reserved9),
        /// DIN registers
        /// - Base address: 0x0C00
        (0x0C00 => din: din_dout::CryptoCellDinRegisters),
        (0x0C5C => _reserved10),
        /// DOUT registers
        /// - Base address: 0x0D00
        (0x0D00 => dout: din_dout::CryptoCellDoutRegisters),
        (0x0D54 => _reserved11),
        /// Host SRAM registers
        /// - Base address: 0x0F00
        (0x0F00 => host_sram: host_sram::CryptoCellHostSramRegisters),
        (0x0F0C => _reserved12),
        /// ID registers
        /// - Base address: 0x0F10
        (0x0F10 => id: id::CryptoCellIdRegisters),
        (0x1000 => @END),
    }
}

#[derive(Copy, Clone)]
enum DigestAlgorithm {
    Md5 = 0,
    Sha1 = 1,
    Sha224 = 10,
    Sha256 = 2,
}

#[derive(Copy, Clone)]
enum HashMode {
    Invalid,
    Digest(DigestAlgorithm),
    Hmac(DigestAlgorithm),
}

// Indicates which operation has been started on the CryptoCell.
// This is used to dispatch interrupts correctly as they are shared between
// different sub-modules of the CryptoCell.
#[derive(Copy, Clone)]
enum OperationMode {
    Idle,
    Hash,
}

pub struct CryptoCell310<'a> {
    registers: StaticRef<CryptoCellRegisters>,
    power: StaticRef<NordicCC310Registers>,
    usage_count: Cell<usize>,
    current_op: Cell<OperationMode>,
    //alarm: time::Alarm,
    aes_client: OptionalCell<&'a dyn hil::symmetric_encryption::Client<'a>>,
    trng_client: OptionalCell<&'a dyn hil::entropy::Client32>,
    sha256_client: OptionalCell<&'a dyn hil::digest::Client<'a, [u8; 32]>>,
    sha1_client: OptionalCell<&'a dyn hil::digest::Client<'a, [u8; 20]>>,
    md5_client: OptionalCell<&'a dyn hil::digest::Client<'a, [u8; 16]>>,

    // Size of the final digest in u32. Should be at most 8
    hash_digest_size: Cell<u32>,
    hash_algo: Cell<HashMode>,
    hash_ctx: Cell<[u32; 8]>,
    hash_hmac_opad_ctx: Cell<[u32; 8]>,
    hash_total_size: Cell<u64>,
    hash_data_queue: Cell<[u8; 64]>,
    hash_data_buff: Cell<Option<LeasableBuffer<'static, u8>>>,
    hash_digest: Cell<Option<&'static mut [u8; 32]>>,
}

const CC310_BASE: StaticRef<CryptoCellRegisters> =
    unsafe { StaticRef::new(0x5002B000 as *const CryptoCellRegisters) };
const CC310_POWER: StaticRef<NordicCC310Registers> =
    unsafe { StaticRef::new(0x5002A500 as *const NordicCC310Registers) };

// Identification “signature” for CryptoCell. According to the documentation, the value
// held by this register is a fixed value, used by Host driver to verify CryptoCell presence
// at this address.
// This value was read from a CryptoCell-310 on a nRF52840-dongle kit.
const CC310_SIGNATURE: u32 = 0x20E00000;

pub static mut DIGEST: [u8; 32] = [0; 32];

impl<'a> CryptoCell310<'a> {
    pub const fn new() -> Self {
        CryptoCell310 {
            registers: CC310_BASE,
            power: CC310_POWER,
            usage_count: Cell::new(0),
            current_op: Cell::new(OperationMode::Idle),

            aes_client: OptionalCell::empty(),
            trng_client: OptionalCell::empty(),
            sha256_client: OptionalCell::empty(),
            sha1_client: OptionalCell::empty(),
            md5_client: OptionalCell::empty(),

            hash_digest_size: Cell::new(0),
            hash_algo: Cell::new(HashMode::Invalid),
            hash_ctx: Cell::new([0; 8]),
            hash_hmac_opad_ctx: Cell::new([0; 8]),
            hash_total_size: Cell::new(0),
            hash_data_queue: Cell::new([0; 64]),
            hash_data_buff: Cell::new(None),
            hash_digest: Cell::new(None),
        }
    }

    pub fn enable(&self) {
        if self.usage_count.get() == 0 {
            //debug!("[CC310] Starting CRYPTOCELL...");
            self.power.enable.write(bitfields::Task::ENABLE::SET);
            if self.registers.ctrl.undocumented.get() >> 24 != 0xf0 {
                debug!(
                    "Invalid magic value. Expected 0xf0######, got {:#x}\n",
                    self.registers.ctrl.undocumented.get()
                );
            }
            if self.registers.host_rgf.signature.get() != CC310_SIGNATURE {
                debug!(
                    "Invalid CC310 signature. Expected {:#x}, got {:#x}\n",
                    CC310_SIGNATURE,
                    self.registers.host_rgf.signature.get()
                );
            }
            // Make sure everything is set to little endian
            self.registers.host_rgf.endian.write(
                bitfields::RgfEndianness::DOUT_WR_BG::LittleEndian
                    + bitfields::RgfEndianness::DIN_RD_BG::LittleEndian
                    + bitfields::RgfEndianness::DOUT_WR_WBG::LittleEndian
                    + bitfields::RgfEndianness::DIN_RD_WBG::LittleEndian,
            );
            // Always start the clock for DMA engine. It's too hard to keep
            // track of which submodule needs DMA otherwise.
            self.registers
                .misc
                .dma_clk_enable
                .write(bitfields::Task::ENABLE::SET);
            self.registers.host_rgf.interrupt_mask.write(
                bitfields::Interrupts::SRAM_TO_DIN::CLEAR
                    + bitfields::Interrupts::DOUT_TO_SRAM::CLEAR
                    + bitfields::Interrupts::MEM_TO_DIN::CLEAR
                    + bitfields::Interrupts::DOUT_TO_MEM::CLEAR
                    + bitfields::Interrupts::AXI_ERROR::SET
                    + bitfields::Interrupts::PKA_EXP::SET
                    + bitfields::Interrupts::RNG::SET
                    + bitfields::Interrupts::SYM_DMA_COMPLETED::CLEAR,
            );
        }
        self.usage_count.set(self.usage_count.get() + 1);
    }

    pub fn disable(&self) {
        if self.usage_count.get() == 0 {
            return;
        }
        //debug!("[CC310] Switching off CRYPTOCELL");
        self.usage_count.set(self.usage_count.get() - 1);
        if self.usage_count.get() == 0 {
            self.registers.host_rgf.interrupt_mask.set(0);
            self.power.enable.write(bitfields::Task::ENABLE::CLEAR);
            self.registers
                .misc
                .dma_clk_enable
                .write(bitfields::Task::ENABLE::CLEAR);
        }
    }

    pub fn handle_interrupt(&self) {
        // mbedTLS is using CryptoCell in blocking mode. In Tock we need to use
        // it in an asynchronous way to comply with the design choices.
        // In addition to that, interrupts are shared between the blocks (AES, SHA, etc.) so
        // we need to check which submodule triggered an interrupt as well as which mode the
        // CryptoCell is currently running to dispatch the interrupts accordingly.
        let regs = &self.registers.host_rgf;
        let intrs = regs.interrupts.extract();

        if intrs.is_set(bitfields::Interrupts::SRAM_TO_DIN) {
            debug!("[CC310] SRAM_TO_DIN interrupt");
            regs.interrupt_clear
                .write(bitfields::Interrupts::SRAM_TO_DIN::SET);
        }

        if intrs.is_set(bitfields::Interrupts::DOUT_TO_SRAM) {
            debug!("[CC310] DOUT_TO_SRAM interrupt");
            regs.interrupt_clear
                .write(bitfields::Interrupts::DOUT_TO_SRAM::SET);
        }

        // A block of data has been fully acquired by the CryptoCell
        if intrs.is_set(bitfields::Interrupts::MEM_TO_DIN) {
            debug!("[CC310] MEM_TO_DIN interrupt");
            regs.interrupt_clear
                .write(bitfields::Interrupts::MEM_TO_DIN::SET);
        }

        // A result data has been fully copied to the chip memory
        if intrs.is_set(bitfields::Interrupts::DOUT_TO_MEM) {
            debug!("[CC310] DOUT_TO_MEM interrupt");
            regs.interrupt_clear
                .write(bitfields::Interrupts::DOUT_TO_MEM::SET);
        }

        if intrs.is_set(bitfields::Interrupts::AXI_ERROR) {
            debug!("[CC310] AXI_ERROR interrupt");
            regs.interrupt_clear
                .write(bitfields::Interrupts::AXI_ERROR::SET);
        }

        // An operationg in the PKA module is finished.
        if intrs.is_set(bitfields::Interrupts::PKA_EXP) {
            debug!("[CC310] PKA_EXP interrupt");
            regs.interrupt_clear
                .write(bitfields::Interrupts::PKA_EXP::SET);
        }

        if intrs.is_set(bitfields::Interrupts::RNG) {
            debug!("[CC310] RNG interrupt");
            /*regs.interrupt_mask.modify(bitfields::Interrupts::RNG::SET);
            let rng_isr = &self.registers.rng.isr.extract();
            regs.rng.icr.set(0xffffffff);
            regs.interrupt_clear.write(bitfields::Interrupts::RNG::SET);
            if rng_isr.is_set(bitfields::RngInterrupt::CRNGT_ERR) {
                // Critical error. Restart the RNG but don't notify the client
                self.restart_trng();
            } else {
                if rng_isr.is_set(bitfields::RngInterrupt::EHR_VALID) {
                    self.read_trng();
                } else {
                    // Non-critical error. Let's collect entropy from the next ROSC
                    self.move_to_next_rosc();
                }
            }*/
            regs.interrupt_mask
                .modify(bitfields::Interrupts::RNG::CLEAR);
        }

        if intrs.is_set(bitfields::Interrupts::SYM_DMA_COMPLETED) {
            debug!("[CC310] SYM_DMA_COMPLETED interrupt");
            regs.interrupt_clear
                .write(bitfields::Interrupts::SYM_DMA_COMPLETED::SET);
        }
    }

    fn get_trng_rand32(&self) -> Option<u32> {
        None
    }

    fn cc_hash_update(&self, data: &[u8], is_last_block: bool) {
        let mut digest = self.hash_ctx.get();
        // Start CryptoCell
        self.enable();
        // TODO(jmichel): move this to async
        while self.registers.ctrl.hash_busy.is_set(bitfields::Busy::BUSY) {}
        while self
            .registers
            .ctrl
            .crypto_busy
            .is_set(bitfields::Busy::BUSY)
        {}
        while self
            .registers
            .din
            .mem_dma_busy
            .is_set(bitfields::Busy::BUSY)
        {}

        // Start HASH module and configure it
        self.current_op.set(OperationMode::Hash);
        self.registers
            .misc
            .hash_clk_enable
            .write(bitfields::Task::ENABLE::SET);
        self.registers
            .ctrl
            .crypto_ctl
            .write(bitfields::CryptoMode::MODE::Hash);
        self.registers
            .hash
            .padding
            .write(bitfields::Task::ENABLE::SET);
        let size = self.hash_total_size.get();
        self.registers.hash.hash_len_lsb.set(size as u32);
        self.registers
            .hash
            .hash_len_msb
            .set(size.wrapping_shr(32) as u32);
        self.registers.hash.control.set(match self.hash_algo.get() {
            HashMode::Digest(alg) | HashMode::Hmac(alg) => alg as u32,
            _ => 2, // By default, pick SHA256
        });

        // Digest must be set backwards because writing to HASH[0]
        // starts computation
        for i in (0..digest.len()).rev() {
            self.registers.hash.hash[i].set(digest[i]);
        }
        while self.registers.ctrl.hash_busy.is_set(bitfields::Busy::BUSY) {}

        // Process data
        if data.len() > 0 {
            if is_last_block {
                self.registers
                    .hash
                    .auto_hw_padding
                    .write(bitfields::Task::ENABLE::SET);
            }
            self.registers.din.src_lli_word0.set(data.as_ptr() as u32);
            self.registers
                .din
                .src_lli_word1
                .write(bitfields::LliWord1::BYTES_NUM.val(data.len() as u32));
            while !self
                .registers
                .host_rgf
                .interrupts
                .is_set(bitfields::Interrupts::MEM_TO_DIN)
            {}
            self.registers
                .host_rgf
                .interrupt_clear
                .write(bitfields::Interrupts::MEM_TO_DIN::SET);
        } else {
            // use DO_PAD to complete padding of previous operation
            self.registers
                .hash
                .pad_config
                .write(hash::PaddingConfig::DO_PAD::SET);
        }
        while self
            .registers
            .ctrl
            .crypto_busy
            .is_set(bitfields::Busy::BUSY)
        {}
        while self
            .registers
            .din
            .mem_dma_busy
            .is_set(bitfields::Busy::BUSY)
        {}

        // Update context and total size
        for i in (0..digest.len()).rev() {
            digest[i] = self.registers.hash.hash[i].get();
        }
        self.hash_ctx.set(digest);
        let new_size: u64 = ((self.registers.hash.hash_len_msb.get() as u64) << 32)
            + (self.registers.hash.hash_len_lsb.get() as u64);
        self.hash_total_size.set(new_size);

        // Disable HASH module
        self.registers
            .hash
            .padding
            .write(bitfields::Task::ENABLE::SET);
        self.registers
            .hash
            .auto_hw_padding
            .write(bitfields::Task::ENABLE::CLEAR);
        self.registers
            .hash
            .pad_config
            .write(hash::PaddingConfig::DO_PAD::CLEAR);
        while self
            .registers
            .ctrl
            .crypto_busy
            .is_set(bitfields::Busy::BUSY)
        {}
        self.registers
            .misc
            .hash_clk_enable
            .write(bitfields::Task::ENABLE::CLEAR);

        self.disable();
    }
}

pub static mut CC310: CryptoCell310<'static> = CryptoCell310::new();
