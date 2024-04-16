use core::cell::RefCell;

use cortex_m::interrupt::{free, Mutex};
use microbit::{
    board::{Buttons, Pins},
    hal::gpiote::Gpiote,
    pac::{self, interrupt, GPIOTE},
};

static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static BUTTON_PRESS: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

pub(crate) fn init_buttons(gpiote: GPIOTE, pins: Pins, buttons: Option<Buttons>) {
    let gpiote = Gpiote::new(gpiote);

    let channel0 = gpiote.channel0();
    let channel1 = gpiote.channel1();

    let p0_02 = pins.p0_02.into_pulldown_input().degrade();

    channel0.input_pin(&p0_02).lo_to_hi().enable_interrupt();
    channel1.input_pin(&p0_02).hi_to_lo().enable_interrupt();

    channel0.reset_events();
    channel1.reset_events();

    if let Some(btn) = buttons {
        let channel2 = gpiote.channel2();
        let channel3 = gpiote.channel3();

        let pin = btn.button_a.into_pulldown_input().degrade();

        // inverted logic
        channel2.input_pin(&pin).hi_to_lo().enable_interrupt();
        channel3.input_pin(&pin).lo_to_hi().enable_interrupt();

        channel2.reset_events();
        channel3.reset_events();
    }

    free(move |cs| {
        *GPIO.borrow(cs).borrow_mut() = Some(gpiote);

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }

        pac::NVIC::unpend(pac::Interrupt::GPIOTE);
    });
}

pub(crate) fn get_pin_state() -> bool {
    free(|cs| {
        return *BUTTON_PRESS.borrow(cs).borrow();
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

            let any_button_released = match (butt_is_released, butt_a_is_released) {
                (false, _) => false,
                (_, false) => false,
                _ => true,
            };

            let any_button_pressed = match (butt_is_pressed, butt_a_is_pressed) {
                (true, _) => true,
                (_, true) => true,
                _ => false,
            };

            let button_state = match (any_button_released, any_button_pressed) {
                (true, _) => false,
                (false, true) => true,
                _ => false,
            };

            gpiote.channel0().reset_events();
            gpiote.channel1().reset_events();

            gpiote.channel2().reset_events();
            gpiote.channel3().reset_events();

            *BUTTON_PRESS.borrow(cs).borrow_mut() = button_state;
        }
    });
}
