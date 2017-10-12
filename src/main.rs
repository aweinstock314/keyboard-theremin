//echo -e '0\n1\n2\n3\n4\n5\n6\n7' | awk '{ split("0,2,4,5,7,9,11,12",a,","); for (i = 0; i < 1; i+= 0.0001) printf("%08X\n", 100*sin(1382*exp((a[$1 % 8]/12)*log(2))*i)) }' | xxd -r -p | aplay -c 2 -f S32_LE -r 16000
extern crate byteorder;

use std::process::{Command, Stdio};
use std::io::Write;
use byteorder::{ByteOrder, LittleEndian};

const NOTES: [f64; 8] = [0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0, 12.0];

fn main() {
    // It seems that aplay labels things backwards, and requires S32_BE to play stuff written with LittleEndian
    //let mut p = Command::new("strace").args(&["aplay", "-c", "2", "-f", "S32_LE", "-r", "16000"]).stdin(Stdio::piped()).spawn().unwrap();
    let mut p = Command::new("aplay").args(&["-c", "2", "-f", "S32_BE", "-r", "16000"]).stdin(Stdio::piped()).spawn().unwrap();
    //let mut p = Command::new("xxd").stdin(Stdio::piped()).spawn().unwrap();
    //let mut p = Command::new("cat").stdin(Stdio::piped()).spawn().unwrap();
    let mut stdin = p.stdin.take().unwrap();
    for n in 0..8 {
        let mut i = 0.0;
        while i < 1.0 {
            let mut tmp = [0u8; 4];
            LittleEndian::write_i32(&mut tmp, (100.0*f64::sin(1382.0*f64::exp(f64::log(2.0, std::f64::consts::E)*NOTES[n%8]/12.0)*i)) as i32);
            stdin.write(&tmp).unwrap();
            //LittleEndian::write_i32(&mut tmp, (100.0*f64::sin(1382.0*f64::exp(f64::log(2.0, std::f64::consts::E)*NOTES[(8-n-1)%8]/12.0)*i)) as i32);
            //stdin.write(&tmp).unwrap();
            i += 0.0001;
        }
    }
}
