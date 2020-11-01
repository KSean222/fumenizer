use std::io::{stdin, stdout, Read, BufReader, Write, BufWriter};
use fumen::{Fumen, CellColor};

struct BaseEightIter<I> {
    iter: I,
    bits: u16,
    bits_length: u8
}

impl<I> BaseEightIter<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            bits: 0,
            bits_length: 0
        }
    }
}

impl<I: Iterator<Item=u8>> Iterator for BaseEightIter<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while self.bits_length + 8 <= 16 {
            if let Some(bits) = self.iter.next() {
                self.bits |= (bits as u16) << self.bits_length;
                self.bits_length += 8;
            } else {
                break;
            }
        }
        if self.bits_length > 0 { 
            self.bits_length = self.bits_length.saturating_sub(3);
            let bits = self.bits as u8 & 0b00000111;
            self.bits >>= 3;
            Some(bits)
        } else {
            None
        }
    }
}

fn main() {
    let mut stdin = BufReader::new(stdin());
    let mut stdout = BufWriter::new(stdout());
    match std::env::args().skip(1).next().as_deref() {
        Some("encode") => {
            let mut fumen = Fumen::default();
            let bytes = stdin.bytes().map(|b| b.unwrap());
            let mut bits = BaseEightIter::new(bytes).peekable();
            while bits.peek().is_some() {
                let page = fumen.add_page();
                let rows = std::iter::once(&mut page.garbage_row)
                    .chain(&mut page.field);
                for row in rows {
                    for cell in row {
                        *cell = match bits.next() {
                            None => CellColor::Empty,
                            Some(0) => CellColor::I,
                            Some(1) => CellColor::L,
                            Some(2) => CellColor::O,
                            Some(3) => CellColor::Z,
                            Some(4) => CellColor::T,
                            Some(5) => CellColor::J,
                            Some(6) => CellColor::S,
                            Some(7) => CellColor::Grey,
                            _ => unreachable!()
                        };
                    }
                }
            }
            stdout.write_all(fumen.encode().as_bytes()).unwrap();
            stdout.flush().unwrap();
        }
        Some("decode") => {
            let mut fumen = String::new();
            stdin.read_to_string(&mut fumen).unwrap();
            let fumen = Fumen::decode(&fumen).unwrap();
            let mut bytes = Vec::new();
            let mut excess_bits = 0;
            'decode: for page in fumen.pages {
               let rows = std::iter::once(&page.garbage_row)
                    .chain(&page.field);
                for row in rows {
                    for &cell in row {
                        let cell = match cell {
                            CellColor::I => 0,
                            CellColor::L => 1,
                            CellColor::O => 2,
                            CellColor::Z => 3,
                            CellColor::T => 4,
                            CellColor::J => 5,
                            CellColor::S => 6,
                            CellColor::Grey => 7,
                            _ => break 'decode
                        };
                        if bytes.is_empty() {
                            bytes.push(0);
                        }
                        let byte = bytes.last_mut().unwrap();
                        *byte |= cell << excess_bits;
                        excess_bits += 3;
                        if excess_bits >= 8 {
                            excess_bits -= 8;
                            bytes.push(cell >> (3 - excess_bits));
                        }
                    }
                }
            }
            if excess_bits > 0 {
                bytes.pop();
            }
            stdout.write_all(&bytes).unwrap();
            stdout.flush().unwrap();
        }
        arg => {
            if let Some(arg) = arg {
                if arg != "help" {
                    eprintln!("Unknown subcommand {}", arg);
                }
            }
            eprintln!("Subcommands: ");
            eprintln!("  help   - Print this message");
            eprintln!("  encode - Encode data from stdin");
            eprintln!("  decode - Decode data from stdin");
        }
    }
}
