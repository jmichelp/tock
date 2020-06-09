//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.
//!
//! Pin Configuration
//! -------------------
//!
//! ### `GPIO`
//!
//! | #  | Pin   | Ix | Header | Arduino |
//! |----|-------|----|--------|---------|
//! | 0  | P1.01 | 33 | P3 1   | D0      |
//! | 1  | P1.02 | 34 | P3 2   | D1      |
//! | 2  | P1.03 | 35 | P3 3   | D2      |
//! | 3  | P1.04 | 36 | P3 4   | D3      |
//! | 4  | P1.05 | 37 | P3 5   | D4      |
//! | 5  | P1.06 | 38 | P3 6   | D5      |
//! | 6  | P1.07 | 39 | P3 7   | D6      |
//! | 7  | P1.08 | 40 | P3 8   | D7      |
//! | 8  | P1.10 | 42 | P4 1   | D8      |
//! | 9  | P1.11 | 43 | P4 2   | D9      |
//! | 10 | P1.12 | 44 | P4 3   | D10     |
//! | 11 | P1.13 | 45 | P4 4   | D11     |
//! | 12 | P1.14 | 46 | P4 5   | D12     |
//! | 13 | P1.15 | 47 | P4 6   | D13     |
//! | 14 | P0.26 | 26 | P4 9   | D14     |
//! | 15 | P0.27 | 27 | P4 10  | D15     |
//!
//! ### `GPIO` / Analog Inputs
//!
//! | #  | Pin        | Header | Arduino |
//! |----|------------|--------|---------|
//! | 16 | P0.03 AIN1 | P2 1   | A0      |
//! | 17 | P0.04 AIN2 | P2 2   | A1      |
//! | 18 | P0.28 AIN4 | P2 3   | A2      |
//! | 19 | P0.29 AIN5 | P2 4   | A3      |
//! | 20 | P0.30 AIN6 | P2 5   | A4      |
//! | 21 | P0.31 AIN7 | P2 6   | A5      |
//! | 22 | P0.02 AIN0 | P4 8   | AVDD    |
//!
//! ### Onboard Functions
//!
//! | Pin   | Header | Function |
//! |-------|--------|----------|
//! | P0.05 | P6 3   | UART RTS |
//! | P0.06 | P6 4   | UART TXD |
//! | P0.07 | P6 5   | UART CTS |
//! | P0.08 | P6 6   | UART RXT |
//! | P0.11 | P24 1  | Button 1 |
//! | P0.12 | P24 2  | Button 2 |
//! | P0.13 | P24 3  | LED 1    |
//! | P0.14 | P24 4  | LED 2    |
//! | P0.15 | P24 5  | LED 3    |
//! | P0.16 | P24 6  | LED 4    |
//! | P0.18 | P24 8  | Reset    |
//! | P0.19 | P24 9  | SPI CLK  |
//! | P0.20 | P24 10 | SPI MOSI |
//! | P0.21 | P24 11 | SPI MISO |
//! | P0.24 | P24 14 | Button 3 |
//! | P0.25 | P24 15 | Button 4 |

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]

use capsules::analog_comparator;
use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};

use capsules::virtual_spi::MuxSpiMaster;
use capsules::virtual_digest::VirtualMuxDigest;
use capsules::virtual_hmac::VirtualMuxHmac;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;

#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};
use nrf52840::gpio::Pin;

// The nRF52840DK LEDs (see back of board)
const LED1_PIN: Pin = Pin::P0_13;
const LED2_PIN: Pin = Pin::P0_14;
const LED3_PIN: Pin = Pin::P0_15;
const LED4_PIN: Pin = Pin::P0_16;

// The nRF52840DK buttons (see back of board)
const BUTTON1_PIN: Pin = Pin::P0_11;
const BUTTON2_PIN: Pin = Pin::P0_12;
const BUTTON3_PIN: Pin = Pin::P0_24;
const BUTTON4_PIN: Pin = Pin::P0_25;
const BUTTON_RST_PIN: Pin = Pin::P0_18;

const UART_RTS: Option<Pin> = Some(Pin::P0_05);
const UART_TXD: Pin = Pin::P0_06;
const UART_CTS: Option<Pin> = Some(Pin::P0_07);
const UART_RXD: Pin = Pin::P0_08;

const SPI_MOSI: Pin = Pin::P0_20;
const SPI_MISO: Pin = Pin::P0_21;
const SPI_CLK: Pin = Pin::P0_19;

const SPI_MX25R6435F_CHIP_SELECT: Pin = Pin::P0_17;
const SPI_MX25R6435F_WRITE_PROTECT_PIN: Pin = Pin::P0_22;
const SPI_MX25R6435F_HOLD_PIN: Pin = Pin::P0_23;

/// Debug Writer
pub mod io;

// Whether to use UART debugging or Segger RTT (USB) debugging.
// - Set to false to use UART.
// - Set to true to use Segger RTT over USB.
const USB_DEBUGGING: bool = true;

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 0x30000] = [0; 0x30000];

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None, None, None, None, None];

static mut CHIP: Option<&'static nrf52840::chip::Chip> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Platform {
    button: &'static capsules::button::Button<'static>,
    pconsole: &'static capsules::process_console::ProcessConsole<
        'static,
        components::process_console::Capability,
    >,
    console: &'static capsules::console::Console<'static>,
    gpio: &'static capsules::gpio::GPIO<'static>,
    led: &'static capsules::led::LED<'static>,
    rng: &'static capsules::rng::RngDriver<'static>,
    temp: &'static capsules::temperature::TemperatureSensor<'static>,
    ipc: kernel::ipc::IPC,
    analog_comparator:
        &'static capsules::analog_comparator::AnalogComparator<'static, nrf52840::acomp::Comparator>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
    >,
    // The nRF52dk does not have the flash chip on it, so we make this optional.
    nonvolatile_storage:
        Option<&'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>>,
    digest: &'static capsules::digest::DigestDriver<'static, VirtualMuxDigest<'static, nrf52840::cryptocell::CryptoCell310<'static>, [u8; 32]>, [u8; 32]>,
    hmac: &'static capsules::hmac::HmacDriver<'static, VirtualMuxHmac<'static, nrf52840::cryptocell::CryptoCell310<'static>, [u8; 32]>, [u8; 32]>,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::rng::DRIVER_NUM => f(Some(self.rng)),
            capsules::temperature::DRIVER_NUM => f(Some(self.temp)),
            capsules::analog_comparator::DRIVER_NUM => f(Some(self.analog_comparator)),
            capsules::nonvolatile_storage_driver::DRIVER_NUM => {
                f(self.nonvolatile_storage.map_or(None, |nv| Some(nv)))
            }
            capsules::hmac::DRIVER_NUM => f(Some(self.hmac)),
            capsules::digest::DRIVER_NUM => f(Some(self.digest)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52840::init();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        components::gpio_component_helper!(
            nrf52840::gpio::GPIOPin,
            0 => &nrf52840::gpio::PORT[Pin::P1_01],
            1 => &nrf52840::gpio::PORT[Pin::P1_02],
            2 => &nrf52840::gpio::PORT[Pin::P1_03],
            3 => &nrf52840::gpio::PORT[Pin::P1_04],
            4 => &nrf52840::gpio::PORT[Pin::P1_05],
            5 => &nrf52840::gpio::PORT[Pin::P1_06],
            6 => &nrf52840::gpio::PORT[Pin::P1_07],
            7 => &nrf52840::gpio::PORT[Pin::P1_08],
            8 => &nrf52840::gpio::PORT[Pin::P1_10],
            9 => &nrf52840::gpio::PORT[Pin::P1_11],
            10 => &nrf52840::gpio::PORT[Pin::P1_12],
            11 => &nrf52840::gpio::PORT[Pin::P1_13],
            12 => &nrf52840::gpio::PORT[Pin::P1_14],
            13 => &nrf52840::gpio::PORT[Pin::P1_15],
            14 => &nrf52840::gpio::PORT[Pin::P0_26],
            15 => &nrf52840::gpio::PORT[Pin::P0_27]
        ),
    )
    .finalize(components::gpio_component_buf!(nrf52840::gpio::GPIOPin));

    let button = components::button::ButtonComponent::new(
        board_kernel,
        components::button_component_helper!(
            nrf52840::gpio::GPIOPin,
            (
                &nrf52840::gpio::PORT[BUTTON1_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //13
            (
                &nrf52840::gpio::PORT[BUTTON2_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //14
            (
                &nrf52840::gpio::PORT[BUTTON3_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ), //15
            (
                &nrf52840::gpio::PORT[BUTTON4_PIN],
                kernel::hil::gpio::ActivationMode::ActiveLow,
                kernel::hil::gpio::FloatingState::PullUp
            ) //16
        ),
    )
    .finalize(components::button_component_buf!(nrf52840::gpio::GPIOPin));

    let led = components::led::LedsComponent::new(components::led_component_helper!(
        nrf52840::gpio::GPIOPin,
        (
            &nrf52840::gpio::PORT[LED1_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &nrf52840::gpio::PORT[LED2_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &nrf52840::gpio::PORT[LED3_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        ),
        (
            &nrf52840::gpio::PORT[LED4_PIN],
            kernel::hil::gpio::ActivationMode::ActiveLow
        )
    ))
    .finalize(components::led_component_buf!(nrf52840::gpio::GPIOPin));

    let chip = static_init!(nrf52840::chip::Chip, nrf52840::chip::new());
    CHIP = Some(chip);

    let hmac_data_buffer = static_init!([u8; 64], [0; 64]);
    let hmac_dest_buffer = static_init!([u8; 32], [0; 32]);
    let mux_hmac = components::hmac::HmacMuxComponent::new(&nrf52840::cryptocell::CC310).finalize(
        components::hmac_mux_component_helper!(nrf52840::cryptocell::CryptoCell310, [u8; 32]),
    );
    let hmac = components::hmac::HmacComponent::new(
        board_kernel,
        &mux_hmac,
        hmac_data_buffer,
        hmac_dest_buffer,
    ).finalize(components::hmac_component_helper!(
        nrf52840::cryptocell::CryptoCell310,
        [u8; 32]
    ));
    let digest_data_buffer = static_init!([u8; 64], [0; 64]);
    let digest_dest_buffer = static_init!([u8; 32], [0; 32]);
    let mux_digest = components::digest::DigestMuxComponent::new(&nrf52840::cryptocell::CC310).finalize(
        components::digest_mux_component_helper!(nrf52840::cryptocell::CryptoCell310, [u8; 32]),
    );
    let digest = components::digest::DigestComponent::new(
        board_kernel,
        &mux_digest,
        digest_data_buffer,
        digest_dest_buffer,
    ).finalize(components::digest_component_helper!(
        nrf52840::cryptocell::CryptoCell310,
        [u8; 32]
    ));

    // Make non-volatile memory writable and activate the reset button
    let uicr = nrf52840::uicr::Uicr::new();

    // Check if we need to erase UICR memory to re-program it
    // This only needs to be done when a bit needs to be flipped from 0 to 1.
    let psel0_reset: u32 = uicr.get_psel0_reset_pin().map_or(0, |pin| pin as u32);
    let psel1_reset: u32 = uicr.get_psel1_reset_pin().map_or(0, |pin| pin as u32);
    let erase_uicr = ((!psel0_reset & (BUTTON_RST_PIN as u32))
        | (!psel1_reset & (BUTTON_RST_PIN as u32))
        | (!(uicr.get_vout() as u32) & (nrf52840::uicr::Regulator0Output::DEFAULT as u32)))
        != 0;

    if erase_uicr {
        nrf52840::nvmc::NVMC.erase_uicr();
    }

    nrf52840::nvmc::NVMC.configure_writeable();
    while !nrf52840::nvmc::NVMC.is_ready() {}

    let mut needs_soft_reset: bool = false;

    // Configure reset pins
    if uicr
        .get_psel0_reset_pin()
        .map_or(true, |pin| pin != BUTTON_RST_PIN)
    {
        uicr.set_psel0_reset_pin(BUTTON_RST_PIN);
        while !nrf52840::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }
    if uicr
        .get_psel1_reset_pin()
        .map_or(true, |pin| pin != BUTTON_RST_PIN)
    {
        uicr.set_psel1_reset_pin(BUTTON_RST_PIN);
        while !nrf52840::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }

    // Configure voltage regulator output
    if uicr.get_vout() != nrf52840::uicr::Regulator0Output::DEFAULT {
        uicr.set_vout(nrf52840::uicr::Regulator0Output::DEFAULT);
        while !nrf52840::nvmc::NVMC.is_ready() {}
        needs_soft_reset = true;
    }

    // Any modification of UICR needs a soft reset for the changes to be taken into account.
    if needs_soft_reset {
        cortexm4::scb::reset();
    }

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&nrf52840::gpio::PORT[LED1_PIN]),
        Some(&nrf52840::gpio::PORT[LED2_PIN]),
        Some(&nrf52840::gpio::PORT[LED3_PIN]),
    );

    let rtc = &nrf52840::rtc::RTC;
    rtc.start();
    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_helper!(nrf52840::rtc::Rtc));
    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(nrf52840::rtc::Rtc));

    // Initialize Segger RTT as early as possible so that any panic beyond this point can use the
    // RTT memory object.
    let mut rtt_memory_refs =
        components::segger_rtt::SeggerRttMemoryComponent::new().finalize(());

    // XXX: This is inherently unsafe as it aliases the mutable reference to rtt_memory. This
    // aliases reference is only used inside a panic handler, which should be OK, but maybe we
    // should use a const reference to rtt_memory and leverage interior mutability instead.
    self::io::set_rtt_memory(&mut *rtt_memory_refs.get_rtt_memory_ptr());

    let rtt = components::segger_rtt::SeggerRttComponent::new(mux_alarm, rtt_memory_refs)
            .finalize(components::segger_rtt_component_helper!(nrf52840::rtc::Rtc));

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux =
        components::console::UartMuxComponent::new(rtt, 115200, dynamic_deferred_caller)
            .finalize(());

    let pconsole =
        components::process_console::ProcessConsoleComponent::new(board_kernel, uart_mux)
            .finalize(());

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let temp = static_init!(
        capsules::temperature::TemperatureSensor<'static>,
        capsules::temperature::TemperatureSensor::new(
            &nrf52840::temperature::TEMP,
            board_kernel.create_grant(&memory_allocation_capability)
        )
    );
    kernel::hil::sensors::TemperatureDriver::set_client(&nrf52840::temperature::TEMP, temp);

    let rng = components::rng::RngComponent::new(board_kernel, &nrf52840::trng::TRNG).finalize(());

    // SPI
    let mux_spi = static_init!(
        MuxSpiMaster<'static, nrf52840::spi::SPIM>,
        MuxSpiMaster::new(&nrf52840::spi::SPIM0)
    );
    hil::spi::SpiMaster::set_client(&nrf52840::spi::SPIM0, mux_spi);
    hil::spi::SpiMaster::init(&nrf52840::spi::SPIM0);
    nrf52840::spi::SPIM0.configure(
        nrf52840::pinmux::Pinmux::new(SPI_MOSI as u32),
        nrf52840::pinmux::Pinmux::new(SPI_MISO as u32),
        nrf52840::pinmux::Pinmux::new(SPI_CLK as u32),
    );

    let nonvolatile_storage: Option<
        &'static capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
    > = {
        // Create a SPI device for the mx25r6435f flash chip.
        let mx25r6435f_spi = static_init!(
            capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
            capsules::virtual_spi::VirtualSpiMasterDevice::new(
                mux_spi,
                &nrf52840::gpio::PORT[SPI_MX25R6435F_CHIP_SELECT]
            )
        );
        // Create an alarm for this chip.
        let mx25r6435f_virtual_alarm = static_init!(
            VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
            VirtualMuxAlarm::new(mux_alarm)
        );
        // Setup the actual MX25R6435F driver.
        let mx25r6435f = static_init!(
            capsules::mx25r6435f::MX25R6435F<
                'static,
                capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
                nrf52840::gpio::GPIOPin,
                VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
            >,
            capsules::mx25r6435f::MX25R6435F::new(
                mx25r6435f_spi,
                mx25r6435f_virtual_alarm,
                &mut capsules::mx25r6435f::TXBUFFER,
                &mut capsules::mx25r6435f::RXBUFFER,
                Some(&nrf52840::gpio::PORT[SPI_MX25R6435F_WRITE_PROTECT_PIN]),
                Some(&nrf52840::gpio::PORT[SPI_MX25R6435F_HOLD_PIN])
            )
        );
        mx25r6435f_spi.set_client(mx25r6435f);
        hil::time::Alarm::set_client(mx25r6435f_virtual_alarm, mx25r6435f);

        pub static mut FLASH_PAGEBUFFER: capsules::mx25r6435f::Mx25r6435fSector =
            capsules::mx25r6435f::Mx25r6435fSector::new();
        let nv_to_page = static_init!(
            capsules::nonvolatile_to_pages::NonvolatileToPages<
                'static,
                capsules::mx25r6435f::MX25R6435F<
                    'static,
                    capsules::virtual_spi::VirtualSpiMasterDevice<'static, nrf52840::spi::SPIM>,
                    nrf52840::gpio::GPIOPin,
                    VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
                >,
            >,
            capsules::nonvolatile_to_pages::NonvolatileToPages::new(
                mx25r6435f,
                &mut FLASH_PAGEBUFFER
            )
        );
        hil::flash::HasClient::set_client(mx25r6435f, nv_to_page);

        let nonvolatile_storage = static_init!(
            capsules::nonvolatile_storage_driver::NonvolatileStorage<'static>,
            capsules::nonvolatile_storage_driver::NonvolatileStorage::new(
                nv_to_page,
                board_kernel.create_grant(&memory_allocation_capability),
                0x60000, // Start address for userspace accessible region
                0x20000, // Length of userspace accessible region
                0,       // Start address of kernel accessible region
                0x60000, // Length of kernel accessible region
                &mut capsules::nonvolatile_storage_driver::BUFFER
            )
        );
        hil::nonvolatile_storage::NonvolatileStorage::set_client(nv_to_page, nonvolatile_storage);
        Some(nonvolatile_storage)
    };

    // Initialize AC using AIN5 (P0.29) as VIN+ and VIN- as AIN0 (P0.02)
    // These are hardcoded pin assignments specified in the driver
    let ac_channels = static_init!(
        [&'static nrf52840::acomp::Channel; 1],
        [&nrf52840::acomp::CHANNEL_AC0,]
    );
    let analog_comparator = static_init!(
        analog_comparator::AnalogComparator<'static, nrf52840::acomp::Comparator>,
        analog_comparator::AnalogComparator::new(&mut nrf52840::acomp::ACOMP, ac_channels)
    );
    nrf52840::acomp::ACOMP.set_client(analog_comparator);

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52840::clock::CLOCK.low_stop();
    nrf52840::clock::CLOCK.high_stop();

    nrf52840::clock::CLOCK.low_set_source(nrf52840::clock::LowClockSource::XTAL);
    nrf52840::clock::CLOCK.low_start();
    nrf52840::clock::CLOCK.high_set_source(nrf52840::clock::HighClockSource::XTAL);
    nrf52840::clock::CLOCK.high_start();
    while !nrf52840::clock::CLOCK.low_started() {}
    while !nrf52840::clock::CLOCK.high_started() {}

    let platform = Platform {
        button: button,
        pconsole: pconsole,
        console: console,
        led: led,
        gpio: gpio,
        rng: rng,
        temp: temp,
        alarm: alarm,
        analog_comparator: analog_comparator,
        nonvolatile_storage: nonvolatile_storage,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        digest: digest,
        hmac: hmac,
    };

    platform.pconsole.start();
    debug!("Initialization complete. Entering main loop\r");
    debug!("{}", &nrf52840::ficr::FICR_INSTANCE);

    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;

        /// End of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _eapps: u8;
    }
    kernel::procs::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            &_sapps as *const u8,
            &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
        ),
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    board_kernel.kernel_loop(&platform, chip, Some(&platform.ipc), &main_loop_capability);
}
