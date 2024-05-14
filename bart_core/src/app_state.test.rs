use crate::AppState;
use smart_leds::RGB8;
use smart_leds::colors;
mod fixtures;

#[test]
fn test_two_rings_lit() {
    let mut app_state = AppState::new();
    app_state.received_http_response(fixtures::json_with_etd("4", "15").to_string());
    let led_buffer = app_state.get_current_led_buffer(0);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_outer_ring::<15>(&mut expected_buffer, colors::YELLOW);
    fill_inner_ring::<4>(&mut expected_buffer, colors::YELLOW);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

#[test]
fn test_shortest_etd_too_long_for_inner_ring() {
    let mut app_state = AppState::new();
    app_state.received_http_response(fixtures::json_with_etd("17", "20").to_string());
    let led_buffer = app_state.get_current_led_buffer(0);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];

    fill_outer_ring::<17>(&mut expected_buffer, colors::YELLOW);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

#[test]
fn test_etd_is_LEAVING() {
    let mut app_state = AppState::new();
    app_state.received_http_response(fixtures::json_with_etd("LEAVING", "15").to_string());
    let led_buffer = app_state.get_current_led_buffer(0);

    let mut expected_buffer: [RGB8; 44] = [colors::BLACK; 44];
    fill_outer_ring::<23>(&mut expected_buffer, colors::YELLOW);
    fill_inner_ring::<15>(&mut expected_buffer, colors::YELLOW);

    assert_eq!(expected_buffer, led_buffer.rgb_buffer);
}

fn fill_outer_ring<const N: usize>(buffer: &mut [RGB8; 44], color: RGB8) {
    buffer[..N].clone_from_slice(&[color; N]);
}

fn fill_inner_ring<const N: usize>(buffer: &mut [RGB8; 44], color: RGB8) {
    buffer[24..24+N].clone_from_slice(&[color; N]);
}
