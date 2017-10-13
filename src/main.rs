//echo -e '0\n1\n2\n3\n4\n5\n6\n7' | awk '{ split("0,2,4,5,7,9,11,12",a,","); for (i = 0; i < 1; i+= 0.0001) printf("%08X\n", 100*sin(1382*exp((a[$1 % 8]/12)*log(2))*i)) }' | xxd -r -p | aplay -c 2 -f S32_LE -r 16000
extern crate byteorder;
extern crate glium;

use std::process::{Command, Stdio};
use std::io::Write;
use byteorder::{ByteOrder, LittleEndian};
use glium::{glutin, Surface};
use std::thread;

const NOTES: [f64; 8] = [0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0, 12.0];

fn square(x: f64) -> f64 {
    if x.fract() < 0.5 { 1.0 } else { 0.0 }
}

fn triangle(x: f64) -> f64 {
    (x.fract()-0.5).abs()
}

fn play_note<F, G>(n: usize, amplitude: f64, freq: f64, wave: F, lfo: Box<G>) where
    F: Fn(f64) -> f64 + Send + 'static, G: Fn(f64, f64) -> f64 + Send + 'static + ?Sized {
    //let mut p = Command::new("strace").args(&["aplay", "-c", "2", "-f", "S32_LE", "-r", "16000"]).stdin(Stdio::piped()).spawn().unwrap();
    //let mut p = Command::new("xxd").stdin(Stdio::piped()).spawn().unwrap();
    //let mut p = Command::new("cat").stdin(Stdio::piped()).spawn().unwrap();
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
                while i < 2.0 {
                    let mut tmp = [0u8; 4];
                    LittleEndian::write_i32(&mut tmp, (lfo(amplitude, i)*wave(freq*f64::exp(f64::log(2.0, std::f64::consts::E)*NOTES[n%8]/12.0)*i)) as i32);
                    let _ = stdin.write(&tmp);
                    i += 0.0001;
                }
            }
        }
    });
}
        

fn main() {
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
    let mut theremin_pos = (0.0, 0.0);
    enum LFOType { Constant, Wave(fn(f64)->f64) }
    let mut lfo = LFOType::Constant;
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
                    let ampmod = (theremin_pos.1 / (height as f64)) + 0.5;
                    let freqmod = 5.0*(theremin_pos.0 / (width as f64)) + 0.5;
                    let lfo_f: Box<Fn(f64, f64) -> f64 + Send + 'static> = match lfo {
                        LFOType::Constant => Box::new(|x, _| x),
                        LFOType::Wave(f) => Box::new(move |x, t| x*f(freqmod*t)),
                    };
                    match event {
                        glutin::WindowEvent::Closed => closed = true,

                        glutin::WindowEvent::MouseMoved { position, .. } => theremin_pos = position,

                        glutin::WindowEvent::ReceivedCharacter('1') => lfo = LFOType::Constant,
                        glutin::WindowEvent::ReceivedCharacter('2') => lfo = LFOType::Wave(f64::sin),

                        glutin::WindowEvent::ReceivedCharacter('a') => play_note(0, ampmod*70.0, freqmod*1382.0, f64::sin, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('s') => play_note(1, ampmod*70.0, freqmod*1382.0, f64::sin, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('d') => play_note(2, ampmod*70.0, freqmod*1382.0, f64::sin, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('f') => play_note(3, ampmod*70.0, freqmod*1382.0, f64::sin, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('g') => play_note(4, ampmod*70.0, freqmod*1382.0, f64::sin, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('h') => play_note(5, ampmod*70.0, freqmod*1382.0, f64::sin, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('j') => play_note(6, ampmod*70.0, freqmod*1382.0, f64::sin, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('k') => play_note(7, ampmod*70.0, freqmod*1382.0, f64::sin, lfo_f),

                        glutin::WindowEvent::ReceivedCharacter('q') => play_note(0, ampmod*150.0, freqmod*700.0, square, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('w') => play_note(1, ampmod*150.0, freqmod*700.0, square, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('e') => play_note(2, ampmod*150.0, freqmod*700.0, square, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('r') => play_note(3, ampmod*150.0, freqmod*700.0, square, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('t') => play_note(4, ampmod*150.0, freqmod*700.0, square, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('y') => play_note(5, ampmod*150.0, freqmod*700.0, square, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('u') => play_note(6, ampmod*150.0, freqmod*700.0, square, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('i') => play_note(7, ampmod*150.0, freqmod*700.0, square, lfo_f),

                        glutin::WindowEvent::ReceivedCharacter('z') => play_note(0, ampmod*200.0, freqmod*800.0, triangle, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('x') => play_note(1, ampmod*200.0, freqmod*800.0, triangle, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('c') => play_note(2, ampmod*200.0, freqmod*800.0, triangle, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('v') => play_note(3, ampmod*200.0, freqmod*800.0, triangle, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('b') => play_note(4, ampmod*200.0, freqmod*800.0, triangle, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('n') => play_note(5, ampmod*200.0, freqmod*800.0, triangle, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter('m') => play_note(6, ampmod*200.0, freqmod*800.0, triangle, lfo_f),
                        glutin::WindowEvent::ReceivedCharacter(',') => play_note(7, ampmod*200.0, freqmod*800.0, triangle, lfo_f),
                        _ => (),
                    }
                },
                _ => (),
            }
        });
    }
}
