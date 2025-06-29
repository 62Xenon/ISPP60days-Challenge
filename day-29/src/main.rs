#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![allow(static_mut_refs, unused_unsafe)]

use arduino_hal::{entry, pac};
use avr_device::asm::sleep;
use panic_halt as _;

// Shared LED for ISR toggle
static mut LED: Option<
    arduino_hal::hal::port::Pin<arduino_hal::hal::port::mode::Output, arduino_hal::hal::port::PB5>,
> = None;

#[entry]
fn main() -> ! {
    // 1) Grab peripherals
    let dp = pac::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // 2) Serial @ 9600 for debug messages
    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
    ufmt::uwriteln!(&mut serial, "Day 29: ready for '#' key").ok();

    // 3) LED on D13
    let mut led = pins.d13.into_output();
    led.set_low();
    unsafe { LED = Some(led) };

    // 4) Keypad rows: only row3 (D6) low → this is the “#” row
    let mut row0 = pins.d9.into_output();
    row0.set_high();
    let mut row1 = pins.d8.into_output();
    row1.set_high();
    let mut row2 = pins.d7.into_output();
    row2.set_high();
    let mut row3 = pins.d6.into_output();
    row3.set_low();

    // 5) Keypad columns: col0=D5, col1=D4, col2=D2(INT0), col3=D3
    let _col0 = pins.d5.into_pull_up_input();
    let _col1 = pins.d4.into_pull_up_input();
    let _col2 = pins.d2.into_pull_up_input(); // <- '#' wired here
    let _col3 = pins.d3.into_pull_up_input(); // <- 'D' wired here

    // 6) Configure INT0 on falling edge (D2)
    dp.EXINT.eicra.write(|w| w.isc0().bits(0b10)); // ISC01=1, ISC00=0
    dp.EXINT.eimsk.write(|w| w.int0().set_bit());

    // 7) Timer2 heartbeat @1Hz
    dp.TC2.tccr2a.write(|w| w.wgm2().ctc());
    dp.TC2.tccr2b.write(|w| w.cs2().prescale_1024());
    dp.TC2.ocr2a.write(|w| w.bits(156)); // ≈1Hz
    dp.TC2.timsk2.write(|w| w.ocie2a().set_bit());

    // 8) Enable interrupts and sleep forever
    unsafe { avr_device::interrupt::enable() };
    loop {
        sleep()
    }
}

// 1Hz heartbeat toggle
#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA() {
    unsafe {
        LED.as_mut().map(|led| led.toggle());
    }
}


#[avr_device::interrupt(atmega328p)]
fn INT0() {
    unsafe {
        LED.as_mut().map(|led| led.toggle());
    }

  
    let dp = unsafe { pac::Peripherals::steal() };
    let pins = unsafe { arduino_hal::pins!(dp) };
    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
    ufmt::uwriteln!(&mut serial, "WAKE").ok();
}

//_____________________________________________________________________________________________

// #![no_std]
// #![no_main]
// #![feature(abi_avr_interrupt)]
// #![allow(static_mut_refs)]

// use arduino_hal::{entry, pac};
// use avr_device::asm::sleep;
// use panic_halt as _;
// // Shared LED handle for ISRs
// static mut LED: Option<
//     arduino_hal::hal::port::Pin<arduino_hal::hal::port::mode::Output, arduino_hal::hal::port::PB5>,
// > = None;

// #[entry]
// fn main() -> ! {
//     // 1) Take peripherals and split pins
//     let dp = pac::Peripherals::take().unwrap();
//     let pins = arduino_hal::pins!(dp);

//     // 2) Initialize serial on D0/D1 at 9600 baud
//     let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
//     // If you want to use rx/tx pins separately, do it after serial is initialized
//     // let rx = pins.d0; let tx = pins.d1;

//     ufmt::uwriteln!(&mut serial, "Day 29: Sleep & wake on '#' key").ok();

//     // 3) Configure LED on D13
//     let mut led = pins.d13.into_output();
//     led.set_low();
//     unsafe { LED = Some(led) };

//     // 4) Matrix keypad rows as outputs, idle HIGH
//     //    Using rowPins = {9, 8, 7, 6}
//     let mut row0 = pins.d9.into_output();
//     row0.set_high();
//     let mut row1 = pins.d8.into_output();
//     row1.set_high();
//     let mut row2 = pins.d7.into_output();
//     row2.set_high();
//     let mut row3 = pins.d6.into_output();
//     row3.set_high();

//     // Columns as inputs with pull-ups
//     //    Using colPins = {5, 4, 3, 2}
//     let _col0 = pins.d5.into_pull_up_input(); // detects keys 1,4,7,*
//     let _col1 = pins.d4.into_pull_up_input(); // detects 2,5,8,0
//     let _col2 = pins.d3.into_pull_up_input(); // detects 3,6,9,# → will use INT1 if needed
//     let _col3 = pins.d2.into_pull_up_input(); // detects A,B,C,D → this is INT0/# wake

//     // 5) Configure INT0 on falling edge
//     dp.EXINT.eicra.write(|w| w.isc0().bits(0b10));
//     dp.EXINT.eimsk.write(|w| w.int0().set_bit());

//     // 6) Timer2 CTC at ~1 Hz for heartbeat
//     dp.TC2.tccr2a.write(|w| w.wgm2().ctc());
//     dp.TC2.tccr2b.write(|w| w.cs2().prescale_1024());
//     dp.TC2.ocr2a.write(|w| w.bits(156)); // 16MHz/1024/156 ≈ 1Hz
//     dp.TC2.timsk2.write(|w| w.ocie2a().set_bit());

//     // 7) Enable global interrupts
//     unsafe { avr_device::interrupt::enable() };

//     // 8) Main sleep loop
//     loop {
//         sleep();
//         ufmt::uwriteln!(&mut serial, "Woke up!").ok();
//     }
// }

// /// Heartbeat: Timer2 Compare Match A
// #[avr_device::interrupt(atmega328p)]
// fn TIMER2_COMPA() {
//     unsafe {
//         if let Some(led) = LED.as_mut() {
//             led.toggle();
//         }
//     }
// }

// /// Wake on '#' key: INT0
// #[avr_device::interrupt(atmega328p)]
// fn INT0() {
//     unsafe {
//         if let Some(led) = LED.as_mut() {
//             led.toggle();
//         }
//     }
// }
