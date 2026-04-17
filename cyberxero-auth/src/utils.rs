//! Utility functions for buffer reading and processing.

use std::io::{ErrorKind, Read};

pub fn read_buffer_with_line_processing<R, F, E>(
    mut reader: R,
    mut send_fn: F,
    mut on_error: E,
) -> bool
where
    R: Read,
    F: FnMut(String) -> bool,
    E: FnMut(std::io::Error),
{
    let mut buffer = [0u8; 4096];
    let mut accumulator = Vec::new();
    let mut last_was_cr = false;

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                if !accumulator.is_empty() {
                    let text = String::from_utf8_lossy(&accumulator).into_owned();
                    if !send_fn(text) {
                        return false;
                    }
                }
                break;
            }
            Ok(n) => {
                for &byte in &buffer[..n] {
                    match byte {
                        b'\r' => {
                            // Send on CR, set state to skip potential following LF
                            if !process_chunk(&mut accumulator, &mut send_fn) {
                                return false;
                            }
                            last_was_cr = true;
                        }
                        b'\n' => {
                            if last_was_cr {
                                // This is the second half of \r\n, ignore it
                                last_was_cr = false;
                            } else {
                                // Standalone \n, process it
                                if !process_chunk(&mut accumulator, &mut send_fn) {
                                    return false;
                                }
                            }
                        }
                        _ => {
                            accumulator.push(byte);
                            last_was_cr = false;
                        }
                    }
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => {
                on_error(e);
                break;
            }
        }
    }
    true
}

/// Helper to convert accumulated bytes to String and send.
fn process_chunk<F>(acc: &mut Vec<u8>, send_fn: &mut F) -> bool
where
    F: FnMut(String) -> bool,
{
    // Ensure the output string has a newline since we stripped the delimiter
    acc.push(b'\n');
    let text = String::from_utf8_lossy(acc).into_owned();
    acc.clear();
    send_fn(text)
}
