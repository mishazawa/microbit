use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};
use microbit::{
    board::Pins,
    hal::{
        gpio::{Input, Pin, PullDown},
        prelude::InputPin,
        rtc::RtcInterrupt,
        Rtc,
    },
    pac::{self, interrupt, RTC0},
};

static RTC: Mutex<RefCell<Option<Rtc<pac::RTC0>>>> = Mutex::new(RefCell::new(None));

static PIN_0: Mutex<RefCell<Option<Pin<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static PIN_1: Mutex<RefCell<Option<Pin<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));

static BUTTONS_STATE: Mutex<RefCell<[bool; 3]>> = Mutex::new(RefCell::new([false; 3]));

pub(crate) fn init_polling(rtc: Rtc<RTC0>) {
    free(|cs| {
        *RTC.borrow(cs).borrow_mut() = Some(rtc);
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::RTC0);
        }
        pac::NVIC::unpend(pac::Interrupt::RTC0);
    });
}

pub(crate) fn init_buttons(pins: Pins) {
    free(|cs| {
        let p0_02 = pins.p0_02.into_pulldown_input().degrade();
        let p0_03 = pins.p0_03.into_pulldown_input().degrade();
        *PIN_0.borrow(cs).borrow_mut() = Some(p0_02);
        *PIN_1.borrow(cs).borrow_mut() = Some(p0_03);
    });
}

pub(crate) fn get_buttons_state() -> [bool; 3] {
    free(|cs| {
        return *BUTTONS_STATE.borrow(cs).borrow();
    })
}

#[interrupt]
fn RTC0() {
    free(|cs| {
        let mut pin_0_state = false;
        let mut pin_1_state = false;

        if let Some(pin_0) = PIN_0.borrow(cs).borrow().as_ref() {
            pin_0_state = pin_0.is_high().unwrap();
        }
        if let Some(pin_1) = PIN_1.borrow(cs).borrow().as_ref() {
            pin_1_state = pin_1.is_high().unwrap();
        }

        if let Some(rtc) = RTC.borrow(cs).borrow().as_ref() {
            rtc.reset_event(RtcInterrupt::Tick);
        }
        *BUTTONS_STATE.borrow(cs).borrow_mut() = [pin_0_state, pin_1_state, false];
    })
}
