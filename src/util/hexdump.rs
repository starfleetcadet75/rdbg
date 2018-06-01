use ansi_term::{self, Colour};
use std::io::Read;

#[derive(Copy, Clone, Debug)]
pub enum Format {
    Octal,
    LowerHex,
    UpperHex,
    Binary,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub offset: u64,
    pub body: Vec<u8>,
    pub ascii: Vec<char>,
    pub bytes: u64,
}

impl Line {
    fn new() -> Line {
        Line {
            offset: 0x0,
            body: Vec::new(),
            ascii: Vec::new(),
            bytes: 0x0,
        }
    }
}

fn build_lines(data: &Vec<u8>, column_width: usize, total: &mut usize) -> Vec<Line> {
    let mut lines = vec![];
    let mut line = Line::new();
    let mut column_count = 0;

    // Build vector of lines from the given data
    for byte in data.bytes() {
        line.body.push(byte.unwrap());
        column_count += 1;
        *total += 1;

        if column_width <= column_count {
            lines.push(line);
            line = Line::new();
            column_count = 0;
        }

        if *total == data.len() {
            lines.push(line);
            break;
        }
    }
    lines
}

pub fn dump(data: &Vec<u8>, format: &str) {
    let format = match format {
        "o" => Format::Octal,
        "x" => Format::LowerHex,
        "X" => Format::UpperHex,
        "b" => Format::Binary,
        _ => Format::LowerHex,
    };

    let mut total = 0;
    let column_width = 16;
    let lines = build_lines(&data, column_width, &mut total);

    let mut ascii_line = Line::new();
    let mut column = 0;
    let mut offset = 0;

    // Print out the lines
    for line in lines {
        print!("{:#08x}: ", offset);

        for hex in line.body.iter() {
            offset += 1;
            column += 1;
            print_byte(*hex, format);

            if 31 < *hex && *hex < 127 {
                ascii_line.ascii.push(*hex as char);
            } else {
                ascii_line.ascii.push('.');
            }
        }

        if column < column_width {
            print!("{:<1$}", "", 5 * (column_width - column) as usize);
        }
        column = 0;

        let string: String = ascii_line.ascii.iter().cloned().collect();
        ascii_line = Line::new();
        print!("{}", string);
        println!("");
    }
}

/// Formats the dumped data as either a Rust, C, or Golang array.
pub fn dump_array(data: &Vec<u8>, format: &str) {
    let mut total = 0;
    let column_width = 16;
    let lines = build_lines(&data, column_width, &mut total);

    match format {
        "r" => println!("let ARRAY: [u8; {}] = [", total),
        "c" => println!("unsigned char ARRAY[{}] = {{", total),
        "g" => println!("a := [{}]byte{{", total),
        _ => println!("Unknown array format"),
    }

    let mut index = 0;
    for line in lines {
        print!("    ");

        for hex in line.body.iter() {
            index += 1;

            if index == data.len() && format != "g" {
                print!("{:#02x}", *hex);
            } else {
                print!("{:#02x}, ", *hex);
            }
        }
        println!("");
    }

    match format {
        "r" => println!("{}", "];"),
        "c" => println!("{}", "};"),
        "g" => println!("{}", "}"),
        _ => println!("Unknown array format"),
    }
}

pub fn print_byte(byte: u8, format: Format) {
    let mut color = Colour::Black;

    if 31 < byte && byte < 127 {
        color = Colour::White;
    } else if byte == 0 {
        color = Colour::Red;
    }

    match format {
        Format::Octal => print!("{} ", ansi_term::Style::new().fg(color).paint(octal(byte))),
        Format::LowerHex => print!(
            "{} ",
            ansi_term::Style::new().fg(color).paint(lower_hex(byte))
        ),
        Format::UpperHex => print!(
            "{} ",
            ansi_term::Style::new().fg(color).paint(upper_hex(byte))
        ),
        Format::Binary => print!("{} ", ansi_term::Style::new().fg(color).paint(binary(byte))),
    }
}

fn octal(byte: u8) -> String { format!("{:06o}", byte) }

fn lower_hex(byte: u8) -> String { format!("{:02x}", byte) }

fn upper_hex(byte: u8) -> String { format!("{:02X}", byte) }

fn binary(byte: u8) -> String { format!("{:010b}", byte) }
