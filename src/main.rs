#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use panic_probe as _;
use rp2040_hal as hal;

use hal::pac;

use hd44780_driver::HD44780; // HD44780 互換 LCD ドライバ
use fugit::RateExtU32;  // ファイルの先頭に追加


#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rp2040_hal::entry]
fn main() -> ! {
    info!("Program start!");
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
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

    let mut timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );


    let sda_pin = pins.gpio16.into_pull_up_input().into_function::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio17.into_pull_up_input().into_function::<hal::gpio::FunctionI2C>();
    
    // I2C0 の初期化（400kHz）
    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400_000u32.Hz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    );

    // PCF8574 バックパック付き LCD の I2C アドレス（多くの場合 0x27 または 0x3F）
    let lcd_address = 0x27;

    let mut lcd = HD44780::new_i2c(i2c, lcd_address, &mut timer)
        .unwrap_or_else(|_| core::panic!("LCD init error!"));

    loop {
        timer.delay_ms(2000);

        lcd.clear(&mut timer).unwrap();

        lcd.write_str("Hello, Pico", &mut timer).unwrap();
    
        lcd.set_cursor_pos(40, &mut timer).unwrap();
        lcd.write_str("Rust on RP2040.", &mut timer).unwrap();

        timer.delay_ms(1000);

        lcd.clear(&mut timer).unwrap();

        // test
        lcd.write_bytes(
            &[0x74, 0x65, 0x73, 0x74]
            , &mut timer).unwrap();
        
    }
}
