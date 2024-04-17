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
static PIN_2: Mutex<RefCell<Option<Pin<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static PIN_3: Mutex<RefCell<Option<Pin<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static PIN_4: Mutex<RefCell<Option<Pin<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static PIN_5: Mutex<RefCell<Option<Pin<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static PIN_6: Mutex<RefCell<Option<Pin<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));

static BUTTONS_STATE: Mutex<RefCell<[bool; 7]>> = Mutex::new(RefCell::new([false; 7]));

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
        let p0_02 = pins.p0_02.into_pulldown_input().degrade(); // p0
        let p0_03 = pins.p0_03.into_pulldown_input().degrade(); // p1
        let p0_04 = pins.p0_04.into_pulldown_input().degrade(); // p2
        let p0_10 = pins.p0_10.into_pulldown_input().degrade(); // p8
        let p0_17 = pins.p0_17.into_pulldown_input().degrade(); // p13
        let p0_01 = pins.p0_01.into_pulldown_input().degrade(); // p14
        let p0_13 = pins.p0_13.into_pulldown_input().degrade(); // p15

        *PIN_0.borrow(cs).borrow_mut() = Some(p0_02);
        *PIN_1.borrow(cs).borrow_mut() = Some(p0_03);
        *PIN_2.borrow(cs).borrow_mut() = Some(p0_04);
        *PIN_3.borrow(cs).borrow_mut() = Some(p0_10);
        *PIN_4.borrow(cs).borrow_mut() = Some(p0_17);
        *PIN_5.borrow(cs).borrow_mut() = Some(p0_01);
        *PIN_6.borrow(cs).borrow_mut() = Some(p0_13);
    });
}

pub(crate) fn get_buttons_state() -> [bool; 7] {
    free(|cs| {
        return *BUTTONS_STATE.borrow(cs).borrow();
    })
}

#[interrupt]
fn RTC0() {
    free(|cs| {
        if let (
            Some(pin_0),
            Some(pin_1),
            Some(pin_2),
            Some(pin_3),
            Some(pin_4),
            Some(pin_5),
            Some(pin_6),
        ) = (
            PIN_0.borrow(cs).borrow().as_ref(),
            PIN_1.borrow(cs).borrow().as_ref(),
            PIN_2.borrow(cs).borrow().as_ref(),
            PIN_3.borrow(cs).borrow().as_ref(),
            PIN_4.borrow(cs).borrow().as_ref(),
            PIN_5.borrow(cs).borrow().as_ref(),
            PIN_6.borrow(cs).borrow().as_ref(),
        ) {
            *BUTTONS_STATE.borrow(cs).borrow_mut() = [
                pin_0.is_high().unwrap(),
                pin_1.is_high().unwrap(),
                pin_2.is_high().unwrap(),
                pin_3.is_high().unwrap(),
                pin_4.is_high().unwrap(),
                pin_5.is_high().unwrap(),
                pin_6.is_high().unwrap(),
            ]
        }

        if let Some(rtc) = RTC.borrow(cs).borrow().as_ref() {
            rtc.reset_event(RtcInterrupt::Tick);
        }
    })
}
