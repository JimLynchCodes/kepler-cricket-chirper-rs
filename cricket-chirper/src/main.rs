#![no_std]
#![no_main]

use panic_halt as _;

// Use the entry macro from the HAL, not cortex_m_rt
// This ensures the bootloader is linked correctly
use rp_pico::entry; 
use core::fmt::Write;
use embedded_hal::digital::OutputPin;

use rp_pico::hal::{
    clocks::{init_clocks_and_plls, Clock},
    fugit::RateExtU32,
    pac,
    sio::Sio,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
};

const XTAL_FREQ_HZ: u32 = 12_000_000;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (
        pins.gpio0.into_function(),
        pins.gpio1.into_function(),
    );

    let mut uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(
                115200.Hz(),
                DataBits::Eight,
                None,
                StopBits::One,
            ),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let mut buzzer = pins.gpio4.into_push_pull_output();

    let mut delay = cortex_m::delay::Delay::new(
        core.SYST,
        clocks.system_clock.freq().to_Hz(),
    );

    let _ = writeln!(uart, "cricket chirper starting");

    let mut seed: u32 = 12345;

    loop {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let seconds = (seed % 60) + 1;

        let _ = writeln!(uart, "waiting {} seconds", seconds);

        delay.delay_ms(seconds * 1000);

        let _ = writeln!(uart, "buzzing");

        for _ in 0..500 {
            let _ = buzzer.set_high();
            delay.delay_us(500);
            let _ = buzzer.set_low();
            delay.delay_us(500);
        }

        delay.delay_ms(500);
    }
}