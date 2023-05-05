//! Test the serial interface
//!
//! This example requires you to short (connect) the TX and RX pins.
//#![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
extern crate nb;
extern crate panic_semihosting;

extern crate stm32l4xx_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::qspi::{Qspi, QspiConfig, QspiMode, QspiReadCommand};
use crate::rt::ExceptionFrame;
use core::fmt::Write;

static LED_ON: i32 = 1;
static LED_OFF: i32 = 0;
/* 定义 8 组 LED 闪灯表，其顺序为 R G B */
static _BLINK_TAB: [[i32; 3]; 8] = [
    [LED_ON, LED_ON, LED_ON],
    [LED_OFF, LED_ON, LED_ON],
    [LED_ON, LED_OFF, LED_ON],
    [LED_ON, LED_ON, LED_OFF],
    [LED_OFF, LED_OFF, LED_ON],
    [LED_ON, LED_OFF, LED_OFF],
    [LED_OFF, LED_ON, LED_OFF],
    [LED_OFF, LED_OFF, LED_OFF],
];

#[entry]
fn main() -> ! {
    let mut index;

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);

    // clock configuration using the default settings (all clocks run at 8 MHz)
    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // TRY this alternate clock configuration (clocks run at nearly the maximum frequency)
    let clocks = rcc
        .cfgr
        .sysclk(80.MHz())
        .pclk1(80.MHz())
        .pclk2(80.MHz())
        .freeze(&mut flash.acr, &mut pwr);

    // The Serial API is highly generic
    // TRY the commented out, different pin configurations
    let tx = gpioa.pa9.into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
    // let tx = gpioa.pa2.into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

    let rx = gpioa.pa10.into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
    // let rx = gpioa.pa3.into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

    // TRY using a different USART peripheral here
    let serial = Serial::usart1(dp.USART1, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb2);
    let (mut tx, mut _rx) = serial.split();

    // core::fmt::Write is implemented for tx.
    write!(tx, "Hello, Pandora!\r\n").unwrap();

    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb2);
    let mut led_r = gpioe
        .pe7
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led_b = gpioe
        .pe8
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led_g = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let qspi = {
        let clk = gpioe
            .pe10
            .into_alternate(&mut gpioe.moder, &mut gpioe.otyper, &mut gpioe.afrh);
        let ncs = gpioe
            .pe11
            .into_alternate(&mut gpioe.moder, &mut gpioe.otyper, &mut gpioe.afrh);
        let io_0 = gpioe
            .pe12
            .into_alternate(&mut gpioe.moder, &mut gpioe.otyper, &mut gpioe.afrh);
        let io_1 = gpioe
            .pe13
            .into_alternate(&mut gpioe.moder, &mut gpioe.otyper, &mut gpioe.afrh);
        let io_2 = gpioe
            .pe14
            .into_alternate(&mut gpioe.moder, &mut gpioe.otyper, &mut gpioe.afrh);
        let io_3 = gpioe
            .pe15
            .into_alternate(&mut gpioe.moder, &mut gpioe.otyper, &mut gpioe.afrh);
        Qspi::new(
            dp.QUADSPI,
            (clk, ncs, io_0, io_1, io_2, io_3),
            &mut rcc.ahb3,
            QspiConfig::default().clock_prescaler(201),
        ) //Added due to missing OSPEEDR register changes in Qspi
    };
    let get_id_command = QspiReadCommand {
        instruction: Some((0x9f, QspiMode::SingleChannel)),
        address: None,
        alternative_bytes: None,
        dummy_cycles: 0,
        data_mode: QspiMode::SingleChannel,
        receive_length: 3,
        double_data_rate: false,
    };
    let mut id_arr: [u8; 3] = [0; 3];
    qspi.transfer(get_id_command, &mut id_arr).unwrap();
    write!(tx, "SPI ID:{:?}\r\n", id_arr).unwrap();

    // Get the delay provider.
    let mut timer = Delay::new(cp.SYST, clocks);
    index = 0;
    loop {
        let r: i32 = _BLINK_TAB[index][0];
        let g: i32 = _BLINK_TAB[index][1];
        let b: i32 = _BLINK_TAB[index][2];
        if r == LED_ON {
            led_r.set_high();
        } else {
            led_r.set_low();
        }
        if g == LED_ON {
            led_g.set_high();
        } else {
            led_g.set_low();
        }
        if b == LED_ON {
            led_b.set_high();
        } else {
            led_b.set_low();
        }

        index = index + 1;
        if index == 8 {
            index = 0;
        }
        timer.delay_ms(500 as u32);
        // Echo what is received on the serial link.
        //let received = block!(rx.read()).unwrap();
        //block!(tx.write(received)).ok();
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}