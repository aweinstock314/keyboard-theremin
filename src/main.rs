//echo -e '0\n1\n2\n3\n4\n5\n6\n7' | awk '{ split("0,2,4,5,7,9,11,12",a,","); for (i = 0; i < 1; i+= 0.0001) printf("%08X\n", 100*sin(1382*exp((a[$1 % 8]/12)*log(2))*i)) }' | xxd -r -p | aplay -c 2 -f S32_LE -r 16000
extern crate byteorder;
extern crate glium;

use std::process::{Command, Stdio};
use std::io::Write;
use byteorder::{ByteOrder, LittleEndian};
use glium::{glutin, Surface};
use std::thread;

const NOTES: [f64; 8] = [0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0, 12.0];

fn main() {
    //let mut p = Command::new("strace").args(&["aplay", "-c", "2", "-f", "S32_LE", "-r", "16000"]).stdin(Stdio::piped()).spawn().unwrap();
    //let mut p = Command::new("xxd").stdin(Stdio::piped()).spawn().unwrap();
    //let mut p = Command::new("cat").stdin(Stdio::piped()).spawn().unwrap();
    let play_note = |n: usize| {
        thread::spawn(move || {
            // It seems that aplay labels things backwards, and requires S32_BE to play stuff written with LittleEndian
            let p = Command::new("aplay")
                .args(&["-c", "2", "-f", "S32_BE", "-r", "16000"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            if let Ok(mut p) = p {
                if let Some(mut stdin) = p.stdin.take() {
                    let mut i = 0.0;
                    while i < 1.0 {
                        let mut tmp = [0u8; 4];
                        LittleEndian::write_i32(&mut tmp, (100.0*f64::sin(1382.0*f64::exp(f64::log(2.0, std::f64::consts::E)*NOTES[n%8]/12.0)*i)) as i32);
                        let _ = stdin.write(&tmp);
                        i += 0.0001;
                    }
                }
            }
        });
    };
    //play_note(0);
    /*for n in 0..8 {
        play_note(n);
    }*/
    let mut events_loop = glium::glutin::EventsLoop::new();
    let (width, height) = (1024, 768);
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(width, height)
        .with_title("Keyboard theremin");
    let context = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let mut closed = false;
    while !closed {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        use glium::Rect;
        target.clear(Some(&Rect { left: width/2, bottom: 0, height: height, width: 10 }), Some((1.0, 0.0, 0.0, 1.0)), false, None, None);
        target.clear(Some(&Rect { left: 0, bottom: height/2, height: 10, width: width }), Some((0.0, 1.0, 0.0, 1.0)), false, None, None);
        target.clear(Some(&Rect { left: width/2, bottom: height/2, height: 10, width: 10 }), Some((0.0, 0.0, 1.0, 1.0)), false, None, None);
        target.finish().unwrap();

        events_loop.poll_events(|ev| {
            match ev {
                glutin::Event::WindowEvent { event, .. } => {
                    //println!("{:?}", event);
                    match event {
                        glutin::WindowEvent::Closed => closed = true,
                        glutin::WindowEvent::ReceivedCharacter('a') => play_note(0),
                        glutin::WindowEvent::ReceivedCharacter('s') => play_note(1),
                        glutin::WindowEvent::ReceivedCharacter('d') => play_note(2),
                        glutin::WindowEvent::ReceivedCharacter('f') => play_note(3),
                        glutin::WindowEvent::ReceivedCharacter('g') => play_note(4),
                        glutin::WindowEvent::ReceivedCharacter('h') => play_note(5),
                        glutin::WindowEvent::ReceivedCharacter('j') => play_note(6),
                        glutin::WindowEvent::ReceivedCharacter('k') => play_note(7),
                        _ => (),
                    }
                },
                _ => (),
            }
        });
    }
}
