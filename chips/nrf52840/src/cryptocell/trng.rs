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

use crate::cryptocell::bitfields::*;
use crate::cryptocell::CryptoCell310;
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{
    register_bitfields, register_structs, InMemoryRegister, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::debug;
use kernel::hil;
use kernel::ReturnCode;

register_bitfields! [u32,
    // RNG register bitfields
    /// A bit sets to '1' masks the interrupt
    /// See RngInterrupt for the meaning of the fields.
    RngInterruptMasking [
        EHR_VALID_INT_MASK OFFSET(0) NUMBITS(1),
        AUTOCORR_ERR_INT_MASK OFFSET(1) NUMBITS(1),
        CRNGT_ERR_INT_MASK OFFSET(2) NUMBITS(1),
        VN_ERR_INT_MASK OFFSET(3) NUMBITS(1),
        WATCHDOG_INT_MASK OFFSET(4) NUMBITS(1),
        RNG_DMA_DONE_INT OFFSET(5) NUMBITS(1)
    ],

    RngInterrupt [
        /// Indicates that 192 bits have been collected in the TRNG and are ready to be read
        EHR_VALID OFFSET(0) NUMBITS(1) [],
        /// Indicates Autocorrelation test failed four times in a row. When it is set,
        /// TRNG ceases to function until next reset
        AUTOCORR_ERR OFFSET(1) NUMBITS(1) [],
        /// Indicates CRNGT in the TRNG test failed. Failure occurs when two consecutive
        /// blocks of 16 collected bits are equal.
        CRNGT_ERR OFFSET(2) NUMBITS(1) [],
        /// Indicates Von Neumann error. Error in von Neumann occurs if 32 consecutive
        /// collected bits are identical, ZERO, or ONE
        VN_ERR OFFSET(3) NUMBITS(1) [],
        /// Indicates RNG DMA to SRAM is completed.
        RNG_DMA_DONE OFFSET(5) NUMBITS(1) [],
        /// Indicates completion of reseeding algorithm with no errors
        RESEEDING_DONE OFFSET(16) NUMBITS(1) [],
        /// Indicates completion of instantiation algorithm with no errors
        INSTANTIATION_DONE OFFSET(17) NUMBITS(1) [],
        /// Indicates completion of final update algorithm
        FINAL_UPDATE_DONE OFFSET(18) NUMBITS(1) [],
        /// Indicates that the result of PRNG is valid and ready to be read.
        /// The result can be read from the RNG_READOUT register
        OUTPUT_READY OFFSET(19) NUMBITS(1) [],
        /// Indicates that the reseed counter has reached 2^48, requiring to
        /// run the reseed algorithm before generating new random numbers
        RESEED_CNTR_FULL OFFSET(20) NUMBITS(1) [],
        /// Indicates that the top 40 bits of the reseed counter are set (that is the
        /// reseed counter is larger than 2^48-2^8). This is a recommendation for
        /// running the reseed algorithm before the counter reaches its max value.
        RESEED_CNTR_TOP_40 OFFSET(21) NUMBITS(1) [],
        /// Indicates CRNGT in the PRNG test failed. Failure occurs when two
        /// consecutive results of AES are equal
        PRNG_CRNGT_ERR OFFSET(22) NUMBITS(1) [],
        /// Indicates that the request size counter (which represents how many
        /// generations of random bits in the PRNG have been produced) has
        /// reached 2^12, thus requiring a working state update before
        /// generating new random numbers.
        REQ_SIZE OFFSET(23) NUMBITS(1) [],
        /// Indicates that one of the KAT (Known Answer Tests) tests has failed.
        /// When set, the entire engine ceases to function
        KAT_ERR OFFSET(24) NUMBITS(1) [],
        /// When the KAT_ERR bit is set, these bits represent which Known
        /// Answer Test had failed
        WHICH_KAT_ERR OFFSET(25) NUMBITS(2) [
            FirstTestInstantiation = 0,
            SecondTestInstantiation = 1,
            FirstTestReseeding = 2,
            SecondTestReseeding = 3
        ]
    ],

    TrngConfig [
        /// Defines the length of the oscillator ring (= the number of
        /// inverters) out of four possible selections.
        RND_SRC_SEL OFFSET(0) NUMBITS(2) [],
        /// Secure Output Port selection
        /// NOTE: Secure output is used for direct connection of the RNG
        /// block outputs to an engine input key.
        /// If CryptoCell does not include a HW PRNG - this field should
        /// be set to 1
        SOP_SEL OFFSET(2) NUMBITS(1) [
            Trng = 0,
            Prng = 1
        ]
    ],

    AutocorrelationStats [
        /// Count each time an autocorrelation test starts. Any write to
        /// the register resets the counter. Stops collecting statistics if
        /// one of the counters has reached the limit.
        TRYS OFFSET(0) NUMBITS(14),
        /// Count each time an autocorrelation test fails. Any write to the
        /// register resets the counter. Stops collecting statistics if one
        /// of the counters has reached the limit.
        FAILS OFFSET(14) NUMBITS(8)
    ],

    RngDebugControl [
        /// When this bit is set, the Von-Neumann balancer is bypassed
        /// (including the 32 consecutive bits test).
        /// NOTE: Can only be set while in debug mode.
        /// If TRNG_TESTS_BYPASS_EN HW flag is defined, this bit can be
        /// set while not in debug mode.
        VNC_BYPASS OFFSET(1) NUMBITS(1),
        /// When this bit is set, the CRNGT test in the TRNG is bypassed.
        /// NOTE: Can only be set while in debug mode.
        /// If TRNG_TESTS_BYPASS_EN HW flag is defined, this bit can be
        /// set while not in debug mode
        TRNG_CRNGT_BYPASS OFFSET(2) NUMBITS(1),
        /// When this bit is set, the autocorrelation test in the TRNG
        /// module is bypassed.
        /// NOTE: Can only be set while in debug mode.
        /// If TRNG_TESTS_BYPASS_EN HW flag is defined, this bit can be
        /// set while not in debug mode.
        AUTO_CORRELATE_BYPASS OFFSET(3) NUMBITS(1)
    ],

    RngBusy [
        /// Reflects rng_busy output port which Consists of trng_busy and prng_busy.
        RngBusy OFFSET(0) NUMBITS(1),
        /// Reflects trng_busy
        TrngBusy OFFSET(1) NUMBITS(1),
        /// Reflects prng_busy
        PrngBusy OFFSET(2) NUMBITS(1)
    ],

    RngVersion [
        EHR_WIDTH OFFSET(0) NUMBITS(1) [
            Bits128 = 0,
            Bits192 = 1
        ],
        CRNGT_EXISTS OFFSET(1) NUMBITS(1) [
            NotExist = 0,
            Exists = 1
        ],
        AUTOCORR_EXISTS OFFSET(2) NUMBITS(1) [
            NotExist = 0,
            Exists = 1
        ],
        TRNG_TESTS_BYPASS_EN OFFSET(3) NUMBITS(1) [
            NotEnabled = 0,
            Enabled = 1
        ],
        PRNG_EXISTS OFFSET(4) NUMBITS(1) [
            NotExist = 0,
            Exists = 1
        ],
        KAT_EXISTS OFFSET(5) NUMBITS(1) [
            NotExist = 0,
            Exists = 1
        ],
        RESEEDING_EXISTS OFFSET(6) NUMBITS(1) [
            NotExist = 0,
            Exists = 1
        ],
        RNG_SBOXES OFFSET(7) NUMBITS(1) [
            AesSbox20 = 0,
            AesSbox5 = 1
        ]
    ],

    RngDmaSource [
        SOURCE_SEL0 OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        SOURCE_SEL1 OFFSET(1) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        SOURCE_SEL2 OFFSET(2) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        SOURCE_SEL3 OFFSET(3) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    RngSramAddress [
        ADDRESS OFFSET(0) NUMBITS(11)
    ],

    RngDmaStatus [
        /// Indicates whether DMA is busy
        DMA_BUSY OFFSET(0) NUMBITS(1),
        /// The active ring oscillator length using by DMA
        DMA_SOURCE_SEL OFFSET(1) NUMBITS(2),
        /// Number of samples already collected in the current ring oscillator chain length
        NUM_OF_SAMPLES OFFSET(3) NUMBITS(8)
    ],

    // Internal use. Not actually part of the CryptoCell
    TrngState [
        /// Last used ring oscillator
        ROSC OFFSET(0) NUMBITS(2),
        /// Which half of the driver buffer we're using to store entropy
        HALF OFFSET(2) NUMBITS(1),
        /// Index on the driver buffer that we're returning to the client.
        INDEX OFFSET(4) NUMBITS(4)
    ]
];

register_structs! {
    pub CryptoCellRngRegisters {
        /// Interrupt masking register
        (0x0000 => pub imr: ReadWrite<u32, RngInterruptMasking::Register>),
        /// Sets Interrupt
        (0x0004 => pub isr: ReadOnly<u32, RngInterrupt::Register>),
        /// Clear Interrupt
        (0x0008 => pub icr: WriteOnly<u32, RngInterrupt::Register>),
        /// This register handles the TRNG configuration
        (0x000C => pub config: ReadWrite<u32, TrngConfig::Register>),
        /// This register indicates that the TRNG data is valid
        (0x0010 => pub valid: ReadOnly<u32, Event::Register>),
        /// This register contains the data collected in the TRNG
        (0x0014 => pub ehr_data: [ReadOnly<u32>; 6]),
        /// This register holds the enable signal for the random source
        (0x002C => pub source_enable: ReadWrite<u32, Task::Register>),
        /// Counts clocks between sampling of random bit
        (0x0030 => pub sample_cnt1: ReadWrite<u32>),
        /// Statistics about autocorrelation test activations
        (0x0034 => pub autocorr_statistics: ReadWrite<u32, AutocorrelationStats::Register>),
        /// This register is used to debug the TRNG
        (0x0038 => pub debug_control: ReadWrite<u32, RngDebugControl::Register>),
        (0x003C => _reserved0),
        /// Generate a software reset solely to RNG block
        (0x0040 => pub sw_reset: ReadWrite<u32, Task::Register>),
        (0x0044 => _reserved1),
        /// Defines the RNG in debug mode
        (0x00B4 => pub debug_enable: ReadOnly<u32, Task::Register>),
        /// RNG busy indication
        (0x00B8 => pub busy: ReadOnly<u32, RngBusy::Register>),
        /// Resets the counter fo collected bits in the TRNG
        /// Writing any value to this address resets the bits counter and trng valid registers.
        /// RND_SOURCE_ENABLE register must be unset in order for reset to take place.
        (0x00BC => pub reset_bits_counter: WriteOnly<u32, Task::Register>),
        /// This register holds the RNG version
        (0x00C0 => pub version: ReadOnly<u32, RngVersion::Register>),
        /// Enables the clock for the RNG block
        (0x00C4 => pub clock_enable: WriteOnly<u32, Task::Register>),
        /// Enables the DMA for the RNG block
        /// Writing value 1'b1 enables RNG DMA to SRAM.
        /// The Value is cleared when DMA completes its operation.
        (0x00C8 => pub dma_enable: ReadWrite<u32, Task::Register>),
        /// This register defines which ring-oscillator length should be used when using the RNG DMA
        (0x00CC => pub dma_src_mask: ReadWrite<u32, RngDmaSource::Register>),
        /// This register defines the start address of the DMA for the TRNG
        (0x00D0 => pub dma_sram_addr: ReadWrite<u32, RngSramAddress::Register>),
        /// This register defines the number of 192-bit samples that the DMA collects per RNG configuration
        (0x00D4 => pub dma_samples_count: ReadWrite<u32, Byte::Register>),
        /// This register defines the maximum number of clock cycles per TRNG collection of 192 samples.
        /// If the number of cycles for a collections exceeds this threashold, the TRNG signals an interrupt.
        (0x00D8 => pub watchdog_val: ReadWrite<u32>),
        /// This register holds the RNG DMA status
        (0x00DC => pub dma_status: ReadOnly<u32, RngDmaStatus::Register>),
        (0x00E0 => @END),
    }
}

/*pub struct CryptoCellTrng<'a> {
    client: OptionalCell<&'a dyn hil::entropy::Client32>,
    // We need to always read twice the EHR per ROSC
    randomness: [Cell<u32>; 12],
    // Fake register to keep track where we are at sampling the ROSC.
    state: InMemoryRegister<u32, TrngState::Register>,
}

impl<'a> CryptoCellTrng<'a> {
    pub fn new() -> Self {
        CryptoCellTrng {
            client: OptionalCell::empty(),
            randomness: Default::default(),
            state: InMemoryRegister::new(0),
        }
    }
}
*/
// Sampling rates for each TRNG ring oscillator.
// This is the default configuration.
const CC310_TRNG_SAMPLING: [u32; 4] = [1000, 1000, 500, 0];

struct FipsTrngIter<'a, 'b: 'a>(&'a CryptoCell310<'b>);

impl<'a, 'b> Iterator for FipsTrngIter<'a, 'b> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.0.get_trng_rand32()
    }
}

impl<'a> hil::entropy::Entropy32<'a> for CryptoCell310<'a> {
    fn get(&self) -> ReturnCode {
        debug!("[CC310] entropy::Entropy32::get()");
        //self.start_trng();
        ReturnCode::SUCCESS
    }

    fn cancel(&self) -> ReturnCode {
        debug!("[CC310] entropy::Entropy32::cancel()");
        // TODO: we should be able to cancel but at the moment, return an error.
        ReturnCode::FAIL
    }

    fn set_client(&'a self, client: &'a dyn hil::entropy::Client32) {
        debug!("[CC310] entropy::Entropy32::set_client()");
        self.trng_client.set(client);
    }
}
