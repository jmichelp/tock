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

use kernel::common::registers::register_bitfields;

register_bitfields! [u32,
    // Generic or shared bitfields
    pub Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    pub Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    pub Byte [
        VALUE OFFSET(0) NUMBITS(8)
    ],

    pub Bool [
        VALUE OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ],

    pub Busy [
        /// Asserted when AES_BUSY or DES_BUSY or HASH_BUSY are asserted or when the DIN FIFO is not empty
        BUSY OFFSET(0) NUMBITS(1) [
            Ready = 0,
            Busy = 1
        ]
    ],

    // PKA register bitfields
    pub MemoryMap [
        /// Contains the physical address in memory to map the register to.
        REG OFFSET(1) NUMBITS(10)
    ],

    pub Opcode [
        /// Holds the opreation's tag or the operand C virtual address
        TAG OFFSET(0) NUMBITS(6) [],
        /// Result register
        REG_R OFFSET(6) NUMBITS(6) [],
        /// Operand B
        REG_B OFFSET(12) NUMBITS(6) [],
        /// Operand A
        REG_A OFFSET(18) NUMBITS(6) [],
        /// The length of the operation.
        /// The value serves as a pointer to PKA length register, for example,
        /// if the value is 0, PKA_L0 holds the size of the operation
        LEN OFFSET(24) NUMBITS(3) [],
        /// Defines the PKA operation
        OPCODE OFFSET(27) NUMBITS(5) [
            Terminate = 0,
            Add_Inc = 4,
            Sub_Dec_Neg = 5,
            ModAdd_ModInc = 6,
            ModSub_ModDec_ModNeg = 7,
            And_Tst0_Clr0 = 8,
            Or_Copy_Set0 = 9,
            Xor_Flip0_Invert_Compare = 10,
            Shr0 = 12,
            Shr1 = 13,
            Shl0 = 14,
            Shl1 = 15,
            MulLow = 16,
            ModMul = 17,
            ModMulN = 18,
            ModExp = 19,
            Division = 20,
            Div = 21,
            ModDiv = 22
        ]
    ],

    pub NNpT0T1 [
        /// Virtual address of register N
        N OFFSET(0) NUMBITS(5),
        /// Virtual address of register NP
        NP OFFSET(5) NUMBITS(5),
        /// Virtual address of temporary register number 0
        T0 OFFSET(10) NUMBITS(5),
        /// Virtual address of temporary register number 1
        T1 OFFSET(15) NUMBITS(5)
    ],

    pub PkaStatus [
        /// The most significant 4-bits of the operand updated in shift operation.
        ALU_MSB OFFSET(0) NUMBITS(4),
        /// The least significant 4-bits of the operand updated in shift operation.
        ALU_LSB OFFSET(4) NUMBITS(4),
        /// Indicates the last operation's sign (MSB).
        ALU_SIGN_OUT OFFSET(8) NUMBITS(1),
        /// Holds the carry of the last ALU operation.
        ALU_CARRY OFFSET(9) NUMBITS(1),
        /// Holds the carry of the last Modular operation.
        ALU_CARRY_MOD OFFSET(10) NUMBITS(1),
        /// Indicates the last subtraction operation's sign .
        ALU_SUB_IS_ZERO OFFSET(11) NUMBITS(1),
        /// Indicates if the result of ALU OUT is zero
        ALU_OUT_ZERO OFFSET(12) NUMBITS(1),
        /// Modular overflow flag
        ALU_MODOVRFLW OFFSET(13) NUMBITS(1),
        /// Indication if the division is done by zero
        DIV_BY_ZERO OFFSET(14) NUMBITS(1),
        /// Indicates the Modular inverse of zero
        MODINV_OF_ZERO OFFSET(15) NUMBITS(1),
        /// Opcode of the last operation
        OPCODE OFFSET(16) NUMBITS(5)
    ],

    /// Operation size in bytes
    pub OpSize [
        SIZE OFFSET(0) NUMBITS(13)
    ],

    pub MonitorSelect [
        /// Defines which PKA FSM monitor is being output
        PKA_MON_SELECT OFFSET(0) NUMBITS(4)
    ],

    pub PkaAddress[
        /// Contains the physical address in memory to map the buffer registers
        ADDRESS OFFSET(0) NUMBITS(12)
    ],

    // ChaCha register bitfields.
    pub ChachaControl [
        CHACHA_OR_SALSA OFFSET(0) NUMBITS(1) [
            ChaCha = 0,
            Salsa = 1
        ],
        /// Start init for new Message
        INIT_FROM_HOST OFFSET(1) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        CALC_KEY_FOR_POLY1305 OFFSET(2) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        KEY_LEN OFFSET(3) NUMBITS(1) [
            Bit256 = 0,
            Bit128 = 1
        ],
        /// The core of ChaCha is a hash function which based on rotation operations.
        /// The hash function consist in application of 20 rounds (default value).
        /// In additional, ChaCha have two variants (they work exactly as the
        /// original algorithm): ChaCha20/8 and ChaCha20/12 (using 8 and 12 rounds)
        /// Default value 0 (i.e. 20 rounds)
        NUM_OF_ROUNDS OFFSET(4) NUMBITS(2) [
            Rounds20 = 0,
            Rounds12 = 1,
            Rounds8 = 2
        ],
        /// For new message
        RESET_BLOCK_CNT OFFSET(9) NUMBITS(1) [],
        USE_IV_96BIT OFFSET(10) NUMBITS(1) []
    ],

    pub ChachaFlags [
        /// If this flag is set, the Salsa/ChaCha engine include ChaCha implementation
        CHACHA_EXISTS OFFSET(0) NUMBITS(1),
        /// If this flag is set, the Salsa/ChaCha engine include Salsa implementation
        SALSA_EXISTS OFFSET(1) NUMBITS(1),
        /// If this flag is set, the next matrix calculated when the current one is
        /// written to data output path (same flag for Salsa core)
        FAST_CHACHA OFFSET(2) NUMBITS(1)
    ],

    pub ChachaByteOrder [
        /// Change the words order of the input data
        CHACHA_DIN_WORD_ORDER OFFSET(0) NUMBITS(1) [
            Normal = 0,
            Reverse = 1
        ],
        /// Change the byte order of the input data
        CHACHA_DIN_BYTE_ORDER OFFSET(1) NUMBITS(1) [
            Normal = 0,
            Reverse = 1
        ],
        /// Change the quarter of a matrix order in core
        CHACHA_CORE_MATRIX_LBE_ORDER OFFSET(2) NUMBITS(1) [
            Normal = 0,
            Reverse = 1
        ],
        /// Change the words order of the output data
        CHACHA_DOUT_WORD_ORDER OFFSET(3) NUMBITS(1) [
            Normal = 0,
            Reverse = 1
        ],
        /// Change the byte order of the output data
        CHACHA_DOUT_BYTE_ORDER OFFSET(4) NUMBITS(1) [
            Normal = 0,
            Reverse = 1
        ]
    ],

    pub ChachaDebug [
        FSM_STATE OFFSET(0) NUMBITS(2) [
            Idle = 0,
            Init = 1,
            Rounds = 2,
            Final = 3
        ]
    ],

    // MISC register bitfields
    pub ClockStatus [
        AES_CLK_STATUS OFFSET(0) NUMBITS(1),
        HASH_CLK_STATUS OFFSET(2) NUMBITS(1),
        PKA_CLK_STATUS OFFSET(3) NUMBITS(1),
        CHACHA_CLK_STATUS OFFSET(7) NUMBITS(1),
        DMA_CLK_STATUS OFFSET(8) NUMBITS(1)
    ],

    // CC_CTL register bitfields
    pub CryptoMode [
        /// Determines the active cryptographic engine
        MODE OFFSET(0) NUMBITS(5) [
            Bypass = 0,
            Aes = 1,
            AesToHash = 2,
            AesAndHash = 3,
            Des = 4,
            DesToHash = 5,
            DesAndHash = 6,
            Hash = 7,
            AesMacAndBypass = 9,
            AesToHashAndDout = 10
        ]
    ],

    // HOST_RGF register bitfields
    pub Interrupts [
        /// This interrupt is asserted when all data was delivered to DIN buffer from SRAM
        SRAM_TO_DIN OFFSET(4) NUMBITS(1),
        /// This interrupt is asserted when all data was delivered to SRAM buffer from DOUT
        DOUT_TO_SRAM OFFSET(5) NUMBITS(1),
        /// This interrupt is asserted when all data was delivered to DIN buffer from memory
        MEM_TO_DIN OFFSET(6) NUMBITS(1),
        /// This interrupt is asserted when all data was delivered to memory buffer from DOUT
        DOUT_TO_MEM OFFSET(7) NUMBITS(1),
        AXI_ERROR OFFSET(8) NUMBITS(1),
        /// The PKA end of operation interrupt status
        PKA_EXP OFFSET(9) NUMBITS(1),
        /// The RNG interrupt status
        RNG OFFSET(10) NUMBITS(1),
        /// The GPR interrupt status
        SYM_DMA_COMPLETED OFFSET(11) NUMBITS(1)
    ],

    pub RgfEndianness [
        /// DOUT write endianness
        DOUT_WR_BG OFFSET(3) NUMBITS(1) [
            LittleEndian = 0,
            BigEndian = 1
        ],
        /// DIN write endianness
        DIN_RD_BG OFFSET(7) NUMBITS(1) [
            LittleEndian = 0,
            BigEndian = 1
        ],
        /// DOUT write word endianness
        DOUT_WR_WBG OFFSET(11) NUMBITS(1) [
            LittleEndian = 0,
            BigEndian = 1
        ],
        /// DIN write word endianness
        DIN_RD_WBG OFFSET(15) NUMBITS(1) [
            LittleEndian = 0,
            BigEndian = 1
        ]
    ],

    pub BootFlags [
        SYNTHESIS_CONFIG OFFSET(0) NUMBITS(1),
        LARGE_RKEK_LOCAL OFFSET(1) NUMBITS(1),
        HASH_IN_FUSES_LOCAL OFFSET(2) NUMBITS(1),
        EXT_MEM_SECURED_LOCAL OFFSET(3) NUMBITS(1),
        RKEK_ECC_EXISTS_LOCAL_N OFFSET(5) NUMBITS(1),
        SRAM_SIZE_LOCAL OFFSET(6) NUMBITS(3),
        DSCRPTR_EXISTS_LOCAL OFFSET(9) NUMBITS(1),
        PAU_EXISTS_LOCAL OFFSET(10) NUMBITS(1),
        RNG_EXISTS_LOCAL OFFSET(11) NUMBITS(1),
        PKA_EXISTS_LOCAL OFFSET(12) NUMBITS(1),
        RC4_EXISTS_LOCAL OFFSET(13) NUMBITS(1),
        SHA_512_PRSNT_LOCAL OFFSET(14) NUMBITS(1),
        SHA_256_PRSNT_LOCAL OFFSET(15) NUMBITS(1),
        MD5_PRSNT_LOCAL OFFSET(16) NUMBITS(1),
        HASH_EXISTS_LOCAL OFFSET(17) NUMBITS(1),
        C2_EXISTS_LOCAL OFFSET(18) NUMBITS(1),
        DES_EXISTS_LOCAL OFFSET(19) NUMBITS(1),
        AES_XCBC_MAC_EXISTS_LOCAL OFFSET(20) NUMBITS(1),
        AES_CMAC_EXISTS_LOCAL OFFSET(21) NUMBITS(1),
        AES_CCM_EXISTS_LOCAL OFFSET(22) NUMBITS(1),
        AES_XEX_HW_T_CALC_LOCAL OFFSET(23) NUMBITS(1),
        AES_XEX_EXISTS_LOCAL OFFSET(24) NUMBITS(1),
        CTR_EXISTS_LOCAL OFFSET(25) NUMBITS(1),
        AES_DIN_BYTE_RESOLUTION_LOCAL OFFSET(26) NUMBITS(1),
        TUNNELING_ENB_LOCAL OFFSET(27) NUMBITS(1),
        SUPPORT_256_192_KEY_LOCAL OFFSET(28) NUMBITS(1),
        ONLY_ENCRYPT_LOCAL OFFSET(29) NUMBITS(1),
        AES_EXISTS_LOCAL OFFSET(30) NUMBITS(1)
    ],

    pub CryptoKey [
        /// Select the source of the HW key that is used by the AES engine
        /// The values are taken from the CryptoCell-312 documentation but
        /// according to Nordic's documentation, this bitfield should be 2 bits
        /// and have the following values:
        ///   - K_DR = 0 (device root key)
        ///   - KPRTL = 1 (hard-coded RTL key)
        ///   - Session = 2 (provided session key)
        KEY OFFSET(0) NUMBITS(3) [
            RKEK = 0,
            KRTL = 1,
            KCP = 2,
            KCE = 3,
            KPICV = 4,
            KCEICV = 5
        ]
    ],

    pub IotLcs [
        /// Lifecycle state value. This field is write-once per reset.
        LCS OFFSET(0) NUMBITS(3) [
            Debug = 0,
            Secure = 2
        ],
        /// This field is read-only and indicates if CRYPTOCELL LCS has
        /// been successfully configured since last reset
        LCS_IS_VALID OFFSET(8) NUMBITS(1) [
            Invalid = 0,
            Valid = 1
        ]
    ],

    pub CryptoCellIdle [
        /// Read if CryptoCell is idle
        HOST_CC_IS_IDLE OFFSET(0) NUMBITS(1),
        /// The event that indicates that CryptoCell is idle
        HOST_CC_IS_IDLE_EVENT OFFSET(1) NUMBITS(1),
        /// Symetric flow is busy
        SYM_IS_BUSY OFFSET(2) NUMBITS(1),
        /// AHB state machine is idle
        AHB_IS_IDLE OFFSET(3) NUMBITS(1),
        /// NVM arbitrer is idle
        NVM_ARB_IS_IDLE OFFSET(4) NUMBITS(1),
        /// NVM is idle
        NVM_IS_IDLE OFFSET(5) NUMBITS(1),
        /// Fatal write
        FATAL_WR OFFSET(6) NUMBITS(1),
        /// RNG is idle
        RNG_IS_IDLE OFFSET(7) NUMBITS(1),
        /// PKA is idle
        PKA_IS_IDLE OFFSET(8) NUMBITS(1),
        /// Crypot flow is idle
        CRYPTO_IS_IDLE OFFSET(9) NUMBITS(1)
    ],

    // AHB register bitfields
    pub AhbProt [
        VALUE OFFSET(0) NUMBITS(4)
    ],

    pub AhbHnonsec [
        WRITE OFFSET(0) NUMBITS(1),
        READ OFFSET(1) NUMBITS(1)
    ],

    // DIN and DOUT register bitfields
    pub LliWord1 [
        /// Total number of bytes to read using DMA in this entry
        BYTES_NUM OFFSET(0) NUMBITS(30),
        /// Indicates the first LLI entry
        FIRST OFFSET(30) NUMBITS(1),
        /// Indicates the last LLI entry
        LAST OFFSET(31) NUMBITS(1)
    ],

    pub Endianness [
        ENDIAN OFFSET(0) NUMBITS(1) [
            LittleEndian = 0,
            BigEndian = 1
        ]
    ],

    pub CpuDataSize [
        /// When using external DMA, the size of transmited data in bytes
        /// should be written to this register
        SIZE OFFSET(0) NUMBITS(16)
    ],

    // HOST_SRAM register bitfields
    pub SramAddress [
        ADDR OFFSET(0) NUMBITS(15)
    ],

    // ID register bitfields
    pub PeripheralId0 [
        /// Identification register part number, bits[7:0]
        PART0 OFFSET(0) NUMBITS(8)
    ],

    pub PeripheralId1 [
        /// Identification register part number, bits[11:8]
        PART1 OFFSET(0) NUMBITS(4),
        /// Constant. Should be set to 0x3B to indicate ARM products.
        DES_0_JEP106 OFFSET(4) NUMBITS(4)
    ],

    pub PeripheralId2 [
        /// Constant. Should be set to 0x3B to indicate ARM products.
        DES_1_JEP106 OFFSET(0) NUMBITS(3),
        /// Constant. Should be set to 1 to indicate that a JEDEC assigned value is used.
        JEDEC OFFSET(3) NUMBITS(1),
        /// starts at zero and increments for every new IP release
        REVISION OFFSET(4) NUMBITS(4)
    ],

    pub PeripheralId3 [
        /// Customer Modified, normally zero, but if a partner applies any
        /// changes themselves, they must change this value
        CMOD OFFSET(0) NUMBITS(4),
        /// starts at zero for every Revision, and increments if metal fixes
        /// are applied between 2 IP releases
        REVAND OFFSET(4) NUMBITS(4)
    ],

    pub PeripheralId4 [
        /// Continuation Code. Should be set to 0x4 to indicate ARM products.
        DES_2_JEP106 OFFSET(0) NUMBITS(4)
    ],

    pub ComponentId0 [
        /// Constant. Should be set to 0xD
        PRMBL_0 OFFSET(0) NUMBITS(8)
    ],

    pub ComponentId1 [
        /// Constant. Should be set to 0x0
        PRMBL_1 OFFSET(0) NUMBITS(4),
        /// Constant. Should be set to 0xF for CryptoCell
        CLASS OFFSET(4) NUMBITS(4)
    ],

    pub ComponentId2 [
        /// Constant. Should be set to 0x5
        PRMBL_2 OFFSET(0) NUMBITS(8)
    ],

    pub ComponentId3 [
        /// Constant. Should be set to 0xB1
        PRMBL_3 OFFSET(0) NUMBITS(8)
    ]
];
