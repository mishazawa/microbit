#![no_main]
#![no_std]

mod buttons;

use cortex_m_rt::entry;

use microbit::display::blocking::Display;
use microbit::hal::rtc::RtcInterrupt;
use microbit::hal::time::Hertz;
use microbit::hal::timer::Timer;
use microbit::hal::{gpio, prelude::*, pwm, Clocks, Rtc};
use microbit::Board;
use microbit::{self as _};

use panic_rtt_target as _;

use rtt_target::{rprintln, rtt_init_print};

use crate::buttons::{get_buttons_state, init_buttons, init_polling};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let light_it_all = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];

    let mut period = 0;

    // row, col
    let led_on_states = [
        [0, 1],
        [0, 2],
        [0, 3],
        [0, 4],
        //
        [1, 4],
        [2, 4],
        [3, 4],
        [4, 4],
        //
        [4, 3],
        [4, 2],
        [4, 1],
        [4, 0],
        //
        [3, 0],
        [2, 0],
        [1, 0],
        [0, 0],
    ];

    let delay_ms = 10u32;

    // init clock for speaker
    let _clocks = Clocks::new(board.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_synth()
        .start_lfclk();

    // diable speaker pin
    let mut speaker_pin = board.speaker_pin.into_push_pull_output(gpio::Level::High);
    let _ = speaker_pin.set_low();

    // init pwm and params
    let speaker = pwm::Pwm::new(board.PWM0);
    speaker
        // output the waveform on the speaker pin
        .set_output_pin(pwm::Channel::C0, speaker_pin.degrade())
        // Use prescale by 16 to achive darker sounds
        .set_prescaler(pwm::Prescaler::Div16)
        // Initial frequency
        .set_period(Hertz(1u32))
        // Configure for up and down counter mode
        .set_counter_mode(pwm::CounterMode::UpAndDown)
        // Set maximum duty cycle
        .set_max_duty(32767)
        // enable PWM
        .enable();

    speaker
        .set_seq_refresh(pwm::Seq::Seq0, 0)
        .set_seq_end_delay(pwm::Seq::Seq0, 0);

    // mute on start
    speaker.disable_channel(pwm::Channel::C0);

    // init real time clock
    let mut rtc = Rtc::new(board.RTC0, 511).unwrap();

    rtc.enable_counter();
    rtc.enable_interrupt(RtcInterrupt::Tick, Some(&mut board.NVIC));
    rtc.enable_event(RtcInterrupt::Tick);

    // init rtc polling interrupt
    init_polling(rtc);
    // init buttons
    init_buttons(board.pins);

    loop {
        let buttons_state = get_buttons_state();

        match buttons_state {
            [true, false, _] => {
                speaker.set_period(Hertz(440u32));
            }
            [false, true, _] => {
                speaker.set_period(Hertz(880u32));
            }
            [true, true, _] => {
                speaker.set_period(Hertz(220u32));
            }
            _ => {
                period += 1;
                period %= led_on_states.len();
            }
        }

        match buttons_state {
            [false, false, _] => {
                speaker.disable_channel(pwm::Channel::C0);
            }
            _ => {
                let max_duty = speaker.max_duty();
                speaker.set_duty_on_common(max_duty / 2);
                speaker.enable_channel(pwm::Channel::C0);
            }
        }

        // rprintln!("{}", speaker.period().0);

        let [row, col] = led_on_states[period];

        let mut curr_state = light_it_all.clone();

        curr_state[row][col] = 1;

        display.show(&mut timer, curr_state, delay_ms);
        // clear the display again
        display.clear();
        timer.delay_ms(delay_ms);
    }
}
