use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};
use microbit::{
    board::{Buttons, Pins},
    hal::gpiote::Gpiote,
    pac::{self, interrupt, GPIOTE},
};
use rtt_target::rprintln;

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static BUTTONS_STATE: Mutex<RefCell<[bool; 3]>> = Mutex::new(RefCell::new([false; 3]));

pub(crate) fn init_buttons(gpiote: GPIOTE, pins: Pins, _buttons: Option<Buttons>) {
    let gpiote = Gpiote::new(gpiote);

    let channel0 = gpiote.channel0();
    let channel1 = gpiote.channel1();
    let channel2 = gpiote.channel2();
    let channel3 = gpiote.channel3();

    let p0_02 = pins.p0_02.into_pulldown_input().degrade();
    let p0_03 = pins.p0_03.into_pulldown_input().degrade();

    channel0.input_pin(&p0_02).lo_to_hi().enable_interrupt();
    channel1.input_pin(&p0_02).hi_to_lo().enable_interrupt();

    channel2.input_pin(&p0_03).lo_to_hi().enable_interrupt();
    channel3.input_pin(&p0_03).hi_to_lo().enable_interrupt();

    channel0.reset_events();
    channel1.reset_events();
    channel2.reset_events();
    channel3.reset_events();

    free(move |cs| {
        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }

        pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    });
}

pub(crate) fn get_pin_state() -> [bool; 3] {
    free(|cs| {
        return *BUTTONS_STATE.borrow(cs).borrow();
    })
}

#[interrupt]
fn GPIOTE() {
    free(|cs| {
        if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
            let butt_is_pressed = gpiote.channel0().is_event_triggered();
            let butt_is_released = gpiote.channel1().is_event_triggered();

            let butt_a_is_pressed = gpiote.channel2().is_event_triggered();
            let butt_a_is_released = gpiote.channel3().is_event_triggered();

            let button_0_state = match butt_is_released {
                true => false,
                false => butt_is_pressed,
            };
            let button_1_state = match butt_a_is_released {
                true => false,
                false => butt_a_is_pressed,
            };

            gpiote.channel0().reset_events();
            gpiote.channel1().reset_events();

            gpiote.channel2().reset_events();
            gpiote.channel3().reset_events();

            *BUTTONS_STATE.borrow(cs).borrow_mut() = [button_0_state, button_1_state, false];
        }
    });
}
