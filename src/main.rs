#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{WindowBuilder, Icon};
use winit_input_helper::WinitInputHelper;
use std::cmp::{min, max};
use image::*;
use image::imageops::*;
use rand::seq::SliceRandom;


const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let icon = {
        let icon_img = open("./images/transparentO.png")
            .expect("Problem opening the file")
            .to_rgba8();
        let icon_width = icon_img.width();
        let icon_height = icon_img.height();
        Icon::from_rgba(icon_img.into_raw(), icon_width, icon_height)
    };
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Multidimensional TicTacToe")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_window_icon(icon.ok())
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
            if input.mouse_pressed(0){
                // let mouse_pos = input.mouse();
                match input.mouse() {
                    Some(position) => {
                        let window_scale = window.scale_factor() as f32;
                        let scaled_position = (position.0/window_scale, position.1/window_scale);
                        world.move_at(scaled_position, Cell::O);
                        window.request_redraw();
                    }
                    None => {}
                }
            }

            // Resize the window, shouldnt ever run cuz resize is false atm
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            // Update internal state and request a redraw

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
            let index = ((j * WIDTH + i) * 4) as usize;
            &frame[index..index+4].copy_from_slice(color);
        }
    }
}

fn img(x: u32, y: u32, image: &RgbaImage, frame: &mut [u8]){
    for(i, j, color) in image.enumerate_pixels(){
        let index = (((j+y) * WIDTH + (i+x)) * 4) as usize;
        &frame[index..index+4].copy_from_slice(color.channels());
    }
}

struct World {
    size: usize,
    dimension: usize,
    board: Vec<Cell>,
    x_img: Vec<RgbaImage>,
    o_img: Vec<RgbaImage>,
    total_width: f32,
    square_size: u32,
    stroke_width: u32,
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new(size: usize, dimension:usize) -> Self {
        let mut x_img = Vec::new();
        let mut o_img = Vec::new();
        let stroke_width = 3;
        let total_width = min(WIDTH,HEIGHT) as f32 * 0.9;
        let square_size = ((total_width/size as f32).round() as u32) - stroke_width;
        for i in 0..4 {//there are 4 images for each

            let load_x = open(format!("./images/X{}.png",i)).expect("Problem opening the file");
            x_img.push(resize(&load_x.to_rgba8(),square_size,square_size,FilterType::Triangle));
            // match load_x {
            //     Ok(v) => ,
            //     Err(e) => panic!("Problem opening the file: {:?}", e)
            // };
            let load_o = open(format!("./images/O{}.png",i)).expect("Problem opening the file");
            o_img.push(resize(&load_o.to_rgba8(),square_size,square_size,FilterType::Triangle));
        }

        Self {
            size,
            dimension,
            board: vec![Cell::Empty; size.pow(dimension as u32)],
            x_img,
            o_img,
            total_width,
            square_size,
            stroke_width
        }
    }

    fn move_at(&mut self, mouse:(f32,f32), move_cell: Cell) {
        let origin = {
            let x = (WIDTH/2) as f32 - self.total_width/2.;
            let y = (HEIGHT/2) as f32 - self.total_width/2.;
            (x.round(), y.round())
        };
        let x = (self.size as f32 * (mouse.0 - origin.0) / self.total_width).min(self.size as f32-1.) as usize;
        let y = (self.size as f32 * (mouse.1 - origin.1) / self.total_width).min(self.size as f32-1.) as usize;
        println!("x: {} y: {}",x,y);
        let cell = self.get_cell(&[x,y]).unwrap();
        *cell = move_cell;
    }
    fn get_cell(&mut self, coords:&[usize]) -> Result<&mut Cell, &'static str>{//idk if 'static is necessary?
        if coords.len() == self.dimension {
            let mut index=0;
            for (d, val) in coords.iter().enumerate() {
                index += val*self.size.pow(d as u32);
            }
            Ok(&mut self.board[index])
        } else {
            Err("invalid number of dimensions")
        }
    }
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&mut self, frame: &mut [u8]) {
        let mut rng = &mut rand::thread_rng();
        let black = &[0x00, 0x00, 0x00, 0xff];
        let white = &[0xff, 0xff, 0xff, 0xff];
        let red   = &[0xff, 0x00, 0x00, 0xff];
        let blue  = &[0x00, 0xff, 0x00, 0xff];
        let green = &[0x00, 0x00, 0xff, 0xff];

        let origin = {
            let x = (WIDTH/2) as f32 - self.total_width/2.;
            let y = (HEIGHT/2) as f32 - self.total_width/2.;
            (x.round() as u32,y.round() as u32)
        };
        let square_offset = self.square_size+self.stroke_width;
        rect(0,0,WIDTH,HEIGHT, white, frame);// CLEAR SCREEN
        // rect(origin.0,origin.1,5,5, red, frame);//origin debug point
        for i in 1..self.size as u32 {
            rect(origin.0,
                 origin.1+(square_offset)*i,
                 self.total_width.round() as u32,
                 self.stroke_width,
                 black, frame);//horizontal lines
            rect(origin.0+(square_offset)*i,
                 origin.1,
                 self.stroke_width,
                 self.total_width.round() as u32,
                 black, frame);//vertical lines
        }
        // rect(200,200,100,100, &[0x00, 0x00, 0x00, 0xff], frame);
        for y in 0..self.size as u32 {
            for x in 0..self.size as u32 {
                // let index:usize = (x+y*self.size as u32) as usize;
                // let cell = &self.board[index];
                let cell = self.get_cell(&[x as usize,y as usize]).unwrap();

                match cell {
                    Cell::X => img(origin.0+self.stroke_width+square_offset*x,
                             origin.1+self.stroke_width+square_offset*y,
                             &self.x_img.choose(&mut rng).expect("Image array of size 0"),
                             frame),
                    Cell::O => img(origin.0+self.stroke_width+square_offset*x,
                            origin.1+self.stroke_width+square_offset*y,
                            &self.o_img.choose(&mut rng).expect("Image array of size 0"),
                            frame),
                    Cell::Empty => ()
                }
            }
        }
    }
}
