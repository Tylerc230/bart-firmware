use crate::AppState;
use smart_leds::RGB8;
use smart_leds::colors;
use std::time::{SystemTime, Duration, UNIX_EPOCH};
mod fixtures;
static LED_COLOR: RGB8 = colors::WHITE;



#[test]
fn test_two_rings_lit() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd("4", "15"));
    let led_buffer = app_state.get_current_led_buffer(0);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_inner_ring::<4>(&mut expected_buffer, colors::WHITE);
    fill_outer_ring::<15>(&mut expected_buffer, colors::WHITE);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}
// TODO: need to test case where a second request comes in and only writes to the first train enry
// in the array (ie to we clear to the buffer to None before filling)
#[test]
fn test_two_rings_lit_after_2_min() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd("4", "15"));
    let two_min_micro = 1000000 * 60 * 2;
    let led_buffer = app_state.get_current_led_buffer(two_min_micro);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_inner_ring::<2>(&mut expected_buffer, colors::WHITE);
    fill_outer_ring::<13>(&mut expected_buffer, colors::WHITE);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

#[test]
fn test_two_rings_lit_first_train_left() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd("4", "15"));
    let five_min_micro = 1000000 * 60 * 5;
    let led_buffer = app_state.get_current_led_buffer(five_min_micro);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_inner_ring::<10>(&mut expected_buffer, colors::WHITE);
    fill_outer_ring::<18>(&mut expected_buffer, colors::WHITE);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

#[test]
fn test_shortest_etd_too_long_for_inner_ring() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd("17", "20"));
    let led_buffer = app_state.get_current_led_buffer(0);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];

    fill_outer_ring::<17>(&mut expected_buffer, colors::WHITE);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

#[test]
fn test_etd_is_leaving() {
    let mut app_state = AppState::new(launch_time());
    app_state.received_http_response(fixtures::json_with_etd("Leaving", "15"));
    let led_buffer = app_state.get_current_led_buffer(0);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_inner_ring::<15>(&mut expected_buffer, colors::WHITE);
    fill_outer_ring::<23>(&mut expected_buffer, colors::WHITE);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

fn fill_outer_ring<const N: usize>(buffer: &mut [RGB8; 44], color: RGB8) {
    buffer[..N].clone_from_slice(&[color; N]);
}

fn fill_inner_ring<const N: usize>(buffer: &mut [RGB8; 44], color: RGB8) {
    buffer[24..24+N].clone_from_slice(&[color; N]);
}

fn launch_time() -> SystemTime {
    UNIX_EPOCH + Duration::new(1_000_000_000, 0)
} 
