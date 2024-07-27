use smart_leds::RGB8;
use smart_leds::colors;
pub trait PipelineStep {
    fn render(&self, led_buffer: &mut LEDBuffer);
}

pub struct LEDBuffer {
    pub rgb_buffer: [RGB8; Self::BUFFER_SIZE as usize],
}

impl LEDBuffer {
    const OUTSIDE_RING_SIZE: i32 = 24;
    const INSIDE_RING_SIZE: i32 = 16;
    const CENTER_RING_SIZE: i32 = 4;
    const BUFFER_SIZE: i32 =  LEDBuffer::OUTSIDE_RING_SIZE + LEDBuffer::INSIDE_RING_SIZE + LEDBuffer::CENTER_RING_SIZE;
    pub fn new() -> LEDBuffer {
        LEDBuffer{rgb_buffer: [RGB8::default(); Self::BUFFER_SIZE as usize]}
    }

    pub fn process_pipeline(pipeline: Vec<&mut dyn PipelineStep>) -> LEDBuffer {
        let mut led_buffer = LEDBuffer::new();
        for step in pipeline {
            step.render(&mut led_buffer);
        }
        led_buffer

    }

    fn outside_ring(&mut self) -> &mut [RGB8] {
        &mut self.rgb_buffer[..Self::OUTSIDE_RING_SIZE as usize]
    }

    fn inside_ring(&mut self) -> &mut [RGB8] {
        &mut self.rgb_buffer[Self::OUTSIDE_RING_SIZE as usize ..(Self::OUTSIDE_RING_SIZE + Self::INSIDE_RING_SIZE) as usize]
    }

    fn center_ring(&mut self) -> &mut [RGB8] {
        &mut self.rgb_buffer[(Self::OUTSIDE_RING_SIZE + Self::INSIDE_RING_SIZE) as usize..]
    }

    fn fill_ring(ring: &mut [RGB8], count: i32, value: RGB8) {
        let count = count as usize;
        for led in ring.iter_mut().take(count) {
            *led = value;
        }
    }
}
pub struct ETDLEDs {
    inside_ring_count: i32,
    outside_ring_count: i32,
}

impl ETDLEDs {
    pub fn new() -> ETDLEDs {
        ETDLEDs { inside_ring_count: 0, outside_ring_count: 0 }
    }
    pub fn update(&mut self, etd_mins: &Vec<i32>, elapse_time_microsec: u64) {
        const MICROSEC_PER_MIN: u64 = 60000000;
        let elapse_time_min = i32::try_from(elapse_time_microsec/MICROSEC_PER_MIN).unwrap();
        let current_etd_min: Vec<i32> = etd_mins.iter()
            .map(|etd| etd - elapse_time_min)//subtract time since fetch
            .filter(|etd| *etd > 0i32)//Filter out trains which have already left
            .collect();
        self.outside_ring_count = 0;
        self.inside_ring_count = 0;
        if current_etd_min.is_empty() {
            return;
        }
        let next_train =  current_etd_min[0];
        if next_train > LEDBuffer::INSIDE_RING_SIZE {
            self.outside_ring_count = next_train;
        } else {
            self.inside_ring_count = next_train;
            if current_etd_min.len() >= 2 {
                let next_next_train = current_etd_min[1];
                self.outside_ring_count = next_next_train;
            }
        }
    }
}

impl PipelineStep for ETDLEDs {
    fn render(&self, led_buffer: &mut LEDBuffer) {
        let color = colors::WHITE;
        LEDBuffer::fill_ring(led_buffer.inside_ring(), self.inside_ring_count, color);
        LEDBuffer::fill_ring(led_buffer.outside_ring(), self.outside_ring_count, color);
    }
}

pub struct NetworkAnimation {
}

impl NetworkAnimation {
    pub fn new() -> Self {
        Self { }
    }
}

impl PipelineStep for NetworkAnimation {

    fn render(&self, led_buffer: &mut LEDBuffer) {
        LEDBuffer::fill_ring(led_buffer.center_ring(), 4, colors::CORNFLOWER_BLUE);
    }
}

pub struct Dim {
    scale_value: u8
}

impl Dim {
    pub fn new(scale_value: u8) -> Self {
        Self { scale_value }
    }
}

impl PipelineStep for Dim {
    fn render(&self, led_buffer: &mut LEDBuffer) {
        let dimmed = smart_leds::brightness(led_buffer.rgb_buffer.into_iter(), self.scale_value); 
        for (i, rgb) in dimmed.enumerate() {
            led_buffer.rgb_buffer[i] = rgb;
        }
    }
}


