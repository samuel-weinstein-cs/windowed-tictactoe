#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use std::cmp::{min, max};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new(3,2);



    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update();
            //window.request_redraw();
        }
    });
}

#[derive(Clone)]
enum Cell {
    Empty,
    X,
    O
}

fn rect(x: u32, y: u32, width: u32, height: u32, color: &[u8; 4], frame: &mut [u8]){
    for i in max(0,x) .. min(WIDTH, x+width) {
        for j in max(0,y) .. min(HEIGHT,y+height) {
            let index:usize = ((j * WIDTH + i) * 4) as usize;
            &frame[index..index+4].copy_from_slice(color);
        }
    }
}

struct World {
    size: usize,
    dimension: usize,
    board: Vec<Cell>
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new(size: usize, dimension:usize) -> Self {
        Self {
            size,
            dimension,
            board: vec![Cell::Empty; size.pow(dimension as u32)]
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {

    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        rect(0,0,WIDTH,HEIGHT, &[0xff, 0xff, 0xff, 0xff], frame);// CLEAR SCREEN
        rect(200,200,100,100, &[0x00, 0xff, 0xff, 0xff], frame);// test rectangle

        // for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        //     let x = (i % WIDTH as usize) as i16;
        //     let y = (i / WIDTH as usize) as i16;
        //     let box_size = min(WIDTH,HEIGHT) as f32 * 0.9;
        //     let inside = x >= (WIDTH  as f32 / 2.0 - box_size / 2.0) as i16
        //               && x <  (WIDTH  as f32 / 2.0 + box_size / 2.0) as i16
        //               && y >= (HEIGHT as f32 / 2.0 - box_size / 2.0) as i16
        //               && y <  (HEIGHT as f32 / 2.0 + box_size / 2.0) as i16;
        //     let rgba = if inside {
        //         [0x00, 0x00, 0x00, 0x00]
        //     } else {
        //         [0xff, 0xff, 0xff, 0xff]
        //     };
        //
        //
        //     pixel.copy_from_slice(&rgba);
        // }
    }
}
