#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::PwmPin;
use panic_halt as _;
use rp_pico::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    pwm::{FreeRunning, Pwm2, Slices},
    sio::Sio,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
};
use core::fmt::Write;
use fugit::RateExtU32;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

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

    // Setup UART
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

    // Setup PWM for buzzer on GP4 (PWM2 channel A)
    let mut pwm_slices = Slices::new(pac.PWM, &mut pac.RESETS);
    let pwm = &mut pwm_slices.pwm2;
    pwm.set_ph_correct();
    pwm.enable();

    let channel = &mut pwm.channel_a;
    channel.output_to(pins.gpio4);

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    writeln!(uart, "starting up").unwrap();

    let mut seed: u32 = 12345;

    loop {
        // Random delay 1-60 seconds
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let random_seconds = (seed % 60) + 1;

        writeln!(uart, "Waiting {} seconds...", random_seconds).unwrap();
        delay.delay_ms(random_seconds * 1000);

        writeln!(uart, "buzzing!").unwrap();

        // Play 1kHz tone using PWM
        play_tone(pwm, channel, 1000);
        delay.delay_ms(500);
        stop_tone(channel);

        delay.delay_ms(500);
    }
}

fn play_tone(
    pwm: &mut rp_pico::hal::pwm::Slice<Pwm2, FreeRunning>,
    channel: &mut rp_pico::hal::pwm::Channel<Pwm2, FreeRunning, rp_pico::hal::pwm::A>,
    frequency: u32,
) {
    let sys_clock = 125_000_000;
    let div = (sys_clock / (frequency * 65536)).max(1);
    let top = (sys_clock / (frequency * div)) as u16;
    
    pwm.set_div_int(div as u8);
    pwm.set_top(top);
    channel.set_duty(top / 2); // 50% duty cycle
}

fn stop_tone(
    channel: &mut rp_pico::hal::pwm::Channel<Pwm2, FreeRunning, rp_pico::hal::pwm::A>,
) {
    channel.set_duty(0);
}