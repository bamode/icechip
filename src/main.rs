#![allow(unused)]
use iced::{
    self,
    canvas::{Cursor, Fill, Frame, Geometry, Path, Program, Stroke},
    Application, Canvas, Color, Command, Element, Length, Point, Rectangle, Settings, Size,
};
use chippers::chip::{Chip8, Chip8Message, KeyCode};

use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::JoinHandle;

fn main() -> iced::Result {
    Chip8App::run(Settings { 
        flags: PathBuf::from("../chip-8/IBM Logo.ch8"), 
        ..Settings::default()
    })?;
    Ok(())
}

struct Chip8App {
    chip_handle: JoinHandle<()>,
    key_tx: Sender<KeyCode>,
    chip_rx: Receiver<Chip8Message>,
    disp: [[u8; 32]; 64],
}

#[derive(Debug, Clone)]
enum Message {
    ClearDisplay,
    Display([[u8; 32]; 64]),
}

impl Application for Chip8App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = std::path::PathBuf;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (key_tx, rx) = channel();
        let (mut chip, chip_rx) = Chip8::new(rx);
        chip.load_rom(flags);
        let chip_handle = std::thread::spawn(move || {
            chip.run().unwrap();
        });
        let disp = [[0u8; 32]; 64];
        (Self { chip_handle, key_tx, chip_rx, disp }, Command::none())
    }

    fn title(&self) -> String {
        "IceChip".into()
    }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        let chip_msg = self.chip_rx.try_recv().unwrap_or(Chip8Message::None);
        match chip_msg {
            Chip8Message::None => Command::none(),
            Chip8Message::ClearScreen => {
                Command::none()
            }
            Chip8Message::DrawScreen(d) => {
                println!("{:?}", d);
                self.disp = d;
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        Canvas::new(RectangleGrid { disp: self.disp })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

struct RectangleGrid {
    disp: [[u8; 32]; 64],
}

impl Program<Message> for RectangleGrid {
    fn draw(&self, bounds: Rectangle, _: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());
        frame.fill_rectangle(
            Point { x: 0., y: 0. },
            Size {
                width: bounds.width,
                height: bounds.height,
            },
            Fill {
                color: Color::BLACK,
                ..Fill::default()
            },
        );

        let origin = Point {
            x: bounds.width / 10.,
            y: bounds.height / 10.,
        };
        let mut br = Point {
            x: 4. * bounds.width / 5.,
            y: 4. * bounds.height / 5.,
        };

        // we want to subdivide the region of `Rectangle { origin, br }` into a 64x32 display

        for (i, row) in self.disp.iter().enumerate() {
            for (j, pix) in row.iter().enumerate() {
                if *pix == 1 {
                    frame.fill_rectangle(
                        Point {
                            x: origin.x + i as f32 * br.x / 64.,
                            y: origin.y + j as f32 * br.y / 32.,
                        },
                        Size {
                            width: br.x / 64.,
                            height: br.y / 32.,
                        },
                        Fill {
                            color: Color::from_rgb8(
                                0,
                                (i as f32 * (255. / 64.)).round() as u8,
                                (j as f32 * (255. / 32.)).round() as u8,
                            ),
                            ..Fill::default()
                        },
                    );
                }
            }
        }
        vec![frame.into_geometry()]
    }
}
