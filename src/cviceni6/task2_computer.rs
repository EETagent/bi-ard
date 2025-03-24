use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = serialport::new("/dev/ttyUSB0", 57600)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    let port = Arc::new(Mutex::new(port));
    let reader_port = Arc::clone(&port);

    println!(
        "Press keys to send commands: 'r' for red, 'g' for green, 'b' for blue. Press 'q' to quit."
    );

    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;

    std::thread::spawn(move || {
        loop {
            let mut serial_buf: Vec<u8> = vec![0; 32];
            let bytes_read = reader_port.lock().unwrap().read(serial_buf.as_mut_slice());

            if let Ok(bytes) = bytes_read {
                let received_text = String::from_utf8_lossy(&serial_buf[..bytes])
                    .replace("\n", "")
                    .replace("\r", "");

                if !received_text.is_empty() {
                    println!("Received: {}\r", received_text);
                }
            }

            std::thread::sleep(Duration::from_micros(100));
        }
    });

    for c in stdin.keys() {
        match c? {
            Key::Char('r') => {
                port.lock().unwrap().write_all(b"r")?;
                println!("Sent: r (red)\r\n");
                stdout.flush()?;
            }
            Key::Char('g') => {
                port.lock().unwrap().write_all(b"g")?;
                println!("Sent: g (green)\r\n");
                stdout.flush()?;
            }
            Key::Char('b') => {
                port.lock().unwrap().write_all(b"b")?;
                println!("Sent: b (blue)\r\n");
                stdout.flush()?;
            }
            Key::Char('q') => {
                println!("Quitting...\r\n");
                stdout.flush()?;
                break;
            }
            _ => {}
        }
        stdout.flush()?;
    }

    Ok(())
}
