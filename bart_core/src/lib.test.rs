use crate::AppState;
use smart_leds::RGB8;
use smart_leds::colors;
use std::time::Duration;
mod fixtures;
static LED_COLOR: RGB8 = RGB8 {r: 0x09, g: 0x09, b: 0x09};



#[test]
fn test_two_rings_lit() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd_3_trains("4", "15"));
    let led_buffer = app_state.get_current_led_buffer(0);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_inner_ring::<4>(&mut expected_buffer, LED_COLOR);
    fill_outer_ring::<15>(&mut expected_buffer, LED_COLOR);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}
// TODO: need to test case where a second request comes in and only writes to the first train enry
// in the array (ie to we clear to the buffer to None before filling)
#[test]
fn test_two_rings_lit_after_2_min() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd_3_trains("4", "15"));
    let two_min_micro = 1000000 * 60 * 2;
    let led_buffer = app_state.get_current_led_buffer(two_min_micro);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_inner_ring::<2>(&mut expected_buffer, LED_COLOR);
    fill_outer_ring::<13>(&mut expected_buffer, LED_COLOR);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

#[test]
fn test_two_rings_lit_first_train_left() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd_3_trains("4", "15"));
    let five_min_micro = 1000000 * 60 * 5;
    let led_buffer = app_state.get_current_led_buffer(five_min_micro);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_inner_ring::<10>(&mut expected_buffer, LED_COLOR);
    fill_outer_ring::<18>(&mut expected_buffer, LED_COLOR);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

#[test]
fn test_shortest_etd_too_long_for_inner_ring() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd_3_trains("17", "20"));
    let led_buffer = app_state.get_current_led_buffer(0);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];

    fill_outer_ring::<17>(&mut expected_buffer, LED_COLOR);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

#[test]
fn test_etd_is_leaving() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd_3_trains("Leaving", "15"));
    let led_buffer = app_state.get_current_led_buffer(0);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_inner_ring::<15>(&mut expected_buffer, LED_COLOR);
    fill_outer_ring::<23>(&mut expected_buffer, LED_COLOR);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

#[test]
fn test_next_fetch_time_2_trains() {
    //We should fetch 2 min before next train leaves (4 -2) * 60 = 120
    let mut app_state = AppState::new(launch_time());
    let next_fetch_sec = app_state.received_http_response(fixtures::json_with_etd_2_trains("4", "15"));
    assert_eq!(next_fetch_sec, 120);
}

#[test]
fn test_next_fetch_time_2_trains_beyond_max() {
    //The most time we should wait between fetches is 10 min (600) CLAMP((13 - 2), 10) * 60 = 600
    let mut app_state = AppState::new(launch_time());
    let next_fetch_sec = app_state.received_http_response(fixtures::json_with_etd_2_trains("13", "15"));
    assert_eq!(next_fetch_sec, 600);
}

#[test]
fn test_next_fetch_time_1_train() {
    //If there's only 1 train scheduled, fetch in 5 min
    let mut app_state = AppState::new(launch_time());
    let next_fetch_sec = app_state.received_http_response(fixtures::json_with_etd_1_train("4"));
    assert_eq!(next_fetch_sec, 300);
}

#[test]
fn test_next_fetch_time_3_train() {
    //If there's 3 or more trains, fetch in 10 min
    let mut app_state = AppState::new(launch_time());
    let next_fetch_sec = app_state.received_http_response(fixtures::json_with_etd_3_trains("4", "15"));
    assert_eq!(next_fetch_sec, 600);
}
//TODO test network activity state

#[test]
fn test_should_allow_fetch_on_launch() {
    let app_state = AppState::new(launch_time());
    let one_min_duration = Duration::new(60, 0);
    assert!(app_state.should_perform_fetch(one_min_duration));
}

#[test]
fn test_should_not_allow_fetch_after_10_min() {
    let app_state = AppState::new(launch_time());
    let ten_min_duration = Duration::new(10 * 60, 0);
    assert!(!app_state.should_perform_fetch(ten_min_duration));
}

#[test]
fn test_should_allow_fetch_after_after_motion_sensed() {
    let mut app_state = AppState::new(launch_time());
    let eight_min_duration = Duration::new(8 * 60, 0);
    app_state.motion_sensed(eight_min_duration);
    let ten_min_duration = Duration::new(10 * 60, 0);
    assert!(app_state.should_perform_fetch(ten_min_duration));
}

fn fill_outer_ring<const N: usize>(buffer: &mut [RGB8; 44], color: RGB8) {
    buffer[..N].clone_from_slice(&[color; N]);
}

fn fill_inner_ring<const N: usize>(buffer: &mut [RGB8; 44], color: RGB8) {
    buffer[24..24+N].clone_from_slice(&[color; N]);
}

fn launch_time() -> Duration {
    Duration::new(0, 0)
} 
