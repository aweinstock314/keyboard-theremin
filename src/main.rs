//echo -e '0\n1\n2\n3\n4\n5\n6\n7' | awk '{ split("0,2,4,5,7,9,11,12",a,","); for (i = 0; i < 1; i+= 0.0001) printf("%08X\n", 100*sin(1382*exp((a[$1 % 8]/12)*log(2))*i)) }' | xxd -r -p | aplay -c 2 -f S32_LE -r 16000
extern crate byteorder;
extern crate cpal;
extern crate glium;

use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
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

struct Note<F: ?Sized, G: ?Sized>(usize, f64, f64, Box<F>, Box<G>);

fn play_note<F, G>(Note(n, amplitude, freq, wave, lfo): Note<F,G>) where
    F: Fn(f64) -> f64 + Send + 'static + ?Sized, G: Fn(f64, f64) -> f64 + Send + 'static + ?Sized {
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

fn ui(s: Sender<Note<Fn(f64) -> f64 + Send, Fn(f64, f64) -> f64 + Send>>) {
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
                        LFOType::Wave(f) => Box::new(move |x, t| x*f(5.0*ampmod*t)),
                    };
                    match event {
                        glutin::WindowEvent::Closed => closed = true,

                        glutin::WindowEvent::MouseMoved { position, .. } => theremin_pos = position,

                        glutin::WindowEvent::ReceivedCharacter('1') => lfo = LFOType::Constant,
                        glutin::WindowEvent::ReceivedCharacter('2') => lfo = LFOType::Wave(f64::sin),

                        glutin::WindowEvent::ReceivedCharacter('a') => { let _ = s.send(Note(0, ampmod*70.0, freqmod*700.0, Box::new(f64::sin), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('s') => { let _ = s.send(Note(1, ampmod*70.0, freqmod*700.0, Box::new(f64::sin), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('d') => { let _ = s.send(Note(2, ampmod*70.0, freqmod*700.0, Box::new(f64::sin), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('f') => { let _ = s.send(Note(3, ampmod*70.0, freqmod*700.0, Box::new(f64::sin), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('g') => { let _ = s.send(Note(4, ampmod*70.0, freqmod*700.0, Box::new(f64::sin), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('h') => { let _ = s.send(Note(5, ampmod*70.0, freqmod*700.0, Box::new(f64::sin), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('j') => { let _ = s.send(Note(6, ampmod*70.0, freqmod*700.0, Box::new(f64::sin), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('k') => { let _ = s.send(Note(7, ampmod*70.0, freqmod*700.0, Box::new(f64::sin), lfo_f)); },

                        glutin::WindowEvent::ReceivedCharacter('q') => { let _ = s.send(Note(0, ampmod*150.0, freqmod*700.0, Box::new(square), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('w') => { let _ = s.send(Note(1, ampmod*150.0, freqmod*700.0, Box::new(square), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('e') => { let _ = s.send(Note(2, ampmod*150.0, freqmod*700.0, Box::new(square), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('r') => { let _ = s.send(Note(3, ampmod*150.0, freqmod*700.0, Box::new(square), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('t') => { let _ = s.send(Note(4, ampmod*150.0, freqmod*700.0, Box::new(square), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('y') => { let _ = s.send(Note(5, ampmod*150.0, freqmod*700.0, Box::new(square), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('u') => { let _ = s.send(Note(6, ampmod*150.0, freqmod*700.0, Box::new(square), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('i') => { let _ = s.send(Note(7, ampmod*150.0, freqmod*700.0, Box::new(square), lfo_f)); },

                        glutin::WindowEvent::ReceivedCharacter('z') => { let _ = s.send(Note(0, ampmod*200.0, freqmod*800.0, Box::new(triangle), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('x') => { let _ = s.send(Note(1, ampmod*200.0, freqmod*800.0, Box::new(triangle), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('c') => { let _ = s.send(Note(2, ampmod*200.0, freqmod*800.0, Box::new(triangle), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('v') => { let _ = s.send(Note(3, ampmod*200.0, freqmod*800.0, Box::new(triangle), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('b') => { let _ = s.send(Note(4, ampmod*200.0, freqmod*800.0, Box::new(triangle), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('n') => { let _ = s.send(Note(5, ampmod*200.0, freqmod*800.0, Box::new(triangle), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter('m') => { let _ = s.send(Note(6, ampmod*200.0, freqmod*800.0, Box::new(triangle), lfo_f)); },
                        glutin::WindowEvent::ReceivedCharacter(',') => { let _ = s.send(Note(7, ampmod*200.0, freqmod*800.0, Box::new(triangle), lfo_f)); },
                        _ => (),
                    }
                },
                _ => (),
            }
        });
    }
}
fn aplay_player<F, G>(r: Receiver<Note<F, G>>) where
    F: Fn(f64) -> f64 + Send + 'static + ?Sized, G: Fn(f64, f64) -> f64 + Send + 'static + ?Sized {
    for note in r.iter() {
        play_note(note);
    }
}

fn cpal_player<F, G>(r: Receiver<Note<F, G>>) where
    F: Fn(f64) -> f64 + Send + 'static + ?Sized, G: Fn(f64, f64) -> f64 + Send + 'static + ?Sized {
    let endpoint = cpal::default_endpoint().expect("Failed to get default endpoint");
    let format = endpoint
        .supported_formats()
        .unwrap()
        .next()
        .expect("Failed to get endpoint format")
        .with_max_samples_rate();

    let event_loop = cpal::EventLoop::new();
    let voice_id = event_loop.build_voice(&endpoint, &format).unwrap();
    event_loop.play(voice_id);

    let pending_notes = Arc::new(Mutex::new(vec![]));
    std::thread::spawn({
        let pn = pending_notes.clone();
        move || {
            for note in r.iter() {
                if let Ok(mut guard) = pn.lock() {
                    guard.push((note, 1.0));
                }
            }
        }
    });

    let samples_rate = format.samples_rate.0 as f64;
    let mut sample_clock = 0f64;

    let next_value = || {
        sample_clock = (sample_clock + 1.0) % samples_rate;
        if let Ok(mut guard) = pending_notes.lock() {
            let mut v = std::mem::replace(&mut *guard, vec![]);
            let num_samples = v.len() as f64;
            let mut result = 0.0;
            for (note, mut timeleft) in v.drain(..) {
                if timeleft > 0.0 {
                    {
                        let Note(n, amplitude, freq, ref wave, ref lfo) = note;
                        let i = sample_clock;
                        result += (lfo(amplitude/200.0, i)*wave((freq*f64::exp(f64::log(2.0, std::f64::consts::E)*NOTES[n%8]/12.0)*i)/samples_rate)) / num_samples;
                        timeleft -= 1.0/samples_rate;
                    }
                    guard.push((note, timeleft));
                }
            }
            result as f32
        } else {
            0.0
        }
    };

    fill_buffers(event_loop, format, next_value);
}

fn fill_buffers<F: FnMut() -> f32 + Send>(event_loop: cpal::EventLoop, format: cpal::Format, mut next_value: F) {
    event_loop.run(move |_, buffer| {
        match buffer {
            cpal::UnknownTypeBuffer::U16(mut buffer) => {
                for sample in buffer.chunks_mut(format.channels.len()) {
                    let value = ((next_value() * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },

            cpal::UnknownTypeBuffer::I16(mut buffer) => {
                for sample in buffer.chunks_mut(format.channels.len()) {
                    let value = (next_value() * std::i16::MAX as f32) as i16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },

            cpal::UnknownTypeBuffer::F32(mut buffer) => {
                for sample in buffer.chunks_mut(format.channels.len()) {
                    let value = next_value();
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            },
        }
    });
}


fn main() {
    let (s,r) = channel();
    std::thread::spawn(move || { ui(s); std::process::exit(0) });
    cpal_player(r);
}
