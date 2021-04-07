#![windows_subsystem = "windows"]
extern crate sdl2;

mod cp437;
mod screen;
mod cmd;
mod font;
mod render;
mod subprocess;

use cmd::{Cmd, CmdEvent};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::{
    keyboard::Mod,
    mouse::{MouseButton, MouseWheelDirection},
    pixels::Color,
};
use subprocess::SubProcess;
use std::{convert::TryInto, process::{Command, Stdio}, time::Duration};

const JF_UNFOCUS_AFTER_KEY: u16 = 0b1000_0000_0000_0000;
const JF_ROLL_COLOR_AFTER_KEY: u16 = 0b0100_0000_0000_0000;
const JF_DISALLOW_MORE_THAN_3_DIGITS_ON_LINE: u16 = 0b0010_0000_0000_0000;
const JF_ALWAYS_ON_TOP: u16 = 0b0001_0000_0000_0000;
const JF_SUBSTITUTE: u16 = 0b0000_1000_0000_0000;
const JF_SCROLL_UP: u16 = 0b0000_0100_0000_0000;

pub fn bitflip(mut s: u16, b: u16) -> u16 {
    s ^= b;
    s
}

pub fn main() {
    let mut color_roll = 0;
    let mut joke_bitmap = 0_u16;
    use font::*;
    use render::*;
    use screen::*;
    let mut cmd = Cmd::new();
    let mut screen = Screen::new(0x07);

    let default_font = Font {
        arrangment: FontArrangment::ASCII,
        glyph_size: (8, 16),
        sheet_width: 255,
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let cwd = std::env::current_dir().unwrap();

    // 80x25 text mode res
    let window = video_subsystem
        .window("Command Prompt", 80 * 8, 25 * 16)
        .set_window_flags(0x00000020)
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut texture_creator = canvas.texture_creator();

    canvas.clear();
    canvas.present();
    let mut font_surface = None;
    let mut smiley_surface = None;
    if let Some(mut dir) = std::env::home_dir() {
        let cdir = dir.clone();
        dir.push("WinCmd");
        match std::env::set_current_dir(dir) {
            Ok(_) => {
                font_surface = Some(sdl2::surface::Surface::load_bmp("./font.bmp").unwrap());
                smiley_surface = Some(sdl2::surface::Surface::load_bmp("./smiley.bmp").unwrap());
                std::env::set_current_dir(cwd);
                if std::env::args().count() > 1 {
                    cmd.attach_child(SubProcess::from_args(std::env::args()));
                }
                else {
                    cmd.attach_child(SubProcess::from_cmd("real_cmd"));
                }
                if !cmd.is_handling_subprocess() {
                    cmd.write_stdout("real_cmd.exe not found (you might have run the executable directly or installation is broken");
                }
            }
            Err(e) => {
                cmd.write_stdout(&format!("{:?}: {} (you might have an incorrect installation)", cdir, e));
            }
        }
    }
    else {
         cmd.write_stdout("Incorrect installation: C:\\Users\\%USER% not found");
    }

    
    let mut font_surface = font_surface.unwrap();
    let mut smiley_surface = smiley_surface.unwrap();
 
           
    let mut font_texture = font_surface.as_texture(&texture_creator).unwrap();
    let mut smiley_texture = smiley_surface.as_texture(&texture_creator).unwrap();   
    let mut visual_cmd = VisualCommandLine::new(font_texture, default_font);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut focus_lost = false;
    'running: loop {
        if cmd.is_exited() {
            break;
        }

        if (joke_bitmap & JF_SCROLL_UP) > 0 {
            visual_cmd.scroll_by(-1);
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    cmd.destroy_child();
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => {
                    joke_bitmap = bitflip(joke_bitmap, JF_UNFOCUS_AFTER_KEY);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => {
                    joke_bitmap = bitflip(joke_bitmap, JF_ROLL_COLOR_AFTER_KEY);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F3),
                    ..
                } => {
                    joke_bitmap = bitflip(joke_bitmap, JF_DISALLOW_MORE_THAN_3_DIGITS_ON_LINE);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F4),
                    ..
                } => {
                    joke_bitmap = bitflip(joke_bitmap, JF_ALWAYS_ON_TOP);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F5),
                    ..
                } => {
                    joke_bitmap = bitflip(joke_bitmap, JF_SUBSTITUTE);
                    cmd.trigger_stdout_update();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F6),
                    ..
                } => {
                    joke_bitmap = bitflip(joke_bitmap, JF_SCROLL_UP);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    cmd.destroy_child();
                    cmd.attach_child(SubProcess::from_cmd("real_cmd"));
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    if !focus_lost {
                        cmd.pop_stdin();
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    if !focus_lost {
                        cmd.put_stdout('\n');
                        cmd.flush_stdin();
                    }
                }
                Event::Window { win_event: WindowEvent::Resized(_, _), .. } |
                Event::Window { win_event: WindowEvent::FocusGained, .. } => {
                    // This is needed because for some strange reason
                    // Because DirectX9 device is becoming 'lost'
                    font_texture = font_surface.as_texture(&texture_creator).unwrap();
                    visual_cmd.set_font_texture(font_texture);
                    smiley_texture = smiley_surface.as_texture(&texture_creator).unwrap();   
                }
                Event::MouseWheel { y, .. } => {
                    visual_cmd.scroll_by(-y * 16);
                }
                Event::TextInput { text, .. } => {
                    if !focus_lost {
                        visual_cmd.lock_scroll();
                        for i in text.chars() {
                            let should_put = if (joke_bitmap
                                & JF_DISALLOW_MORE_THAN_3_DIGITS_ON_LINE)
                                > 0
                            {
                                !(i.is_digit(10)
                                    && cmd.get_stdin().chars().filter(|c| c.is_digit(10)).count()
                                        >= 3)
                            } else {
                                true
                            };

                            if should_put {
                                cmd.put_stdin(i);
                            }
                        }
                        if (joke_bitmap & JF_ROLL_COLOR_AFTER_KEY) > 0 {
                            color_roll += 1;
                        }
                    }
                    // We are going to simulate focus here
                    if (joke_bitmap & JF_UNFOCUS_AFTER_KEY) > 0 {
                        focus_lost = true;
                    }
                }
                Event::MouseMotion {
                    xrel, yrel, x, y, ..
                } => {
                    sdl_context
                        .mouse()
                        .capture(visual_cmd.mouse_move(&canvas, (x, y), yrel));
                }
                Event::MouseButtonDown { x, y, .. } => {
                    focus_lost = false;
                    visual_cmd.mouse_press(&canvas, (x, y));
                }
                Event::MouseButtonUp { x, y, .. } => {
                    visual_cmd.mouse_release(&canvas, (x, y));
                }
                _ => {}
            }
        }

        if (joke_bitmap & JF_ALWAYS_ON_TOP) > 0 {
            canvas.window_mut().raise()
        }

        screen.color = ((color_roll % 8) << 4) | ((color_roll + 7) % 8);
        for event in cmd.drain_events() {
            match event {
                CmdEvent::ChildExited => {
                    break 'running;
                },
                CmdEvent::StdoutChanged => {
                    let mut s = cmd.get_stdout().to_string();
                    if (joke_bitmap & JF_SUBSTITUTE) > 0 {
                        s = s.replace("Foreign", "Trusted")
                            .replace("ESTABLISHED", "SECURE")
                            .replace("REFUND", "AIRPLANE")
                            .replace("Refund", "Airplane")
                            .replace("refund", "airplane");
                    }
                    screen.set_text(s);
                }
            }
        }
        cmd.update();

        if !focus_lost {
            visual_cmd.update(canvas.window().size(), &screen);
        }
        visual_cmd.render(&mut canvas, &screen);
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
}
