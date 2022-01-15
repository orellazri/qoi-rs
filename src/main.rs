use clap::Parser;
use std::{
    fs,
    io::{Cursor, Read},
    ops::Sub,
    path::Path,
};

const QOI_HEADER_SIZE: usize = 14;
const QOI_END_MARKER: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 1];
const QOI_END_MARKER_SIZE: usize = QOI_END_MARKER.len();

const QOI_OP_RUN: u8 = 0xc0;
const QOI_OP_INDEX: u8 = 0x00;
const QOI_OP_DIFF: u8 = 0x40;
const QOI_OP_LUMA: u8 = 0x80;
const QOI_OP_RGB: u8 = 0xfe;
const QOI_OP_RGBA: u8 = 0xff;

#[derive(Copy, Clone, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Sub for Color {
    type Output = Color;
    fn sub(self, rhs: Self) -> Self::Output {
        Color {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
            a: self.a - rhs.a,
        }
    }
}

fn write32(bytes: &mut Vec<u8>, data: u32) {
    bytes.push(((data & 0xff000000) >> 24) as u8);
    bytes.push(((data & 0x00ff0000) >> 16) as u8);
    bytes.push(((data & 0x0000ff00) >> 8) as u8);
    bytes.push((data & 0x000000ff) as u8);
}

#[derive(Parser, Debug)]
struct Args {
    input_filename: String,
    width: u32,
    height: u32,
    channels: u8,
}

fn main() {
    let args = Args::parse();
    let input_filename = args.input_filename;
    let width: u32 = args.width;
    let height: u32 = args.height;
    let channels: u8 = args.channels;
    let colorspace: u8 = 1;

    let buffer = fs::read(&input_filename).unwrap();
    let buffer_len = buffer.len();
    let mut buffer = Cursor::new(buffer);

    let mut bytes: Vec<u8> = Vec::with_capacity(
        width as usize * height as usize * (channels as usize + 1)
            + QOI_HEADER_SIZE
            + QOI_END_MARKER_SIZE,
    );

    let mut prev_color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    let mut seen_pixels = [Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    }; 64];

    bytes.extend("qoif".as_bytes());
    write32(&mut bytes, width);
    write32(&mut bytes, height);
    bytes.push(channels);
    bytes.push(colorspace);

    let last_pixel = buffer_len - channels as usize;
    let mut chunk = vec![0; channels as usize];
    let mut run = 0u8;
    for offset in (0..=last_pixel).step_by(channels as usize) {
        buffer.read_exact(&mut chunk).unwrap();

        let color = Color {
            r: chunk[0],
            g: chunk[1],
            b: chunk[2],
            a: if channels == 4 {
                chunk[3]
            } else {
                prev_color.a
            },
        };

        // Check for a run
        if color == prev_color {
            run += 1;

            // Write run when cannot keep track anymore
            if run == 62 || offset == last_pixel {
                bytes.push(QOI_OP_RUN | (run - 1));
                run = 0;
            }
        } else {
            // Write previous run (if any)
            if run > 0 {
                bytes.push(QOI_OP_RUN | (run - 1));
                run = 0;
            }

            // Check if the pixel is in the seen array
            let index_position = ((color.r as usize * 3)
                + (color.g as usize * 5)
                + (color.b as usize) * 7
                + (color.a as usize) * 11)
                % 64;
            if color == seen_pixels[index_position as usize] {
                bytes.push(QOI_OP_INDEX | index_position as u8);
            } else {
                seen_pixels[index_position as usize] = color;

                // Check diffs
                let dr = color.r as i32 - prev_color.r as i32;
                let dg = color.g as i32 - prev_color.g as i32;
                let db = color.b as i32 - prev_color.b as i32;
                let da = color.a as i32 - prev_color.a as i32;

                if da == 0 {
                    let dr_dg = dr - dg;
                    let db_dg = db - dg;

                    if dr >= -2 && dr <= 1 && dg >= -2 && dg <= 1 && db >= -2 && db <= 1 {
                        // Write diff
                        let dr: u8 = (dr + 2).try_into().unwrap();
                        let dg: u8 = (dg + 2).try_into().unwrap();
                        let db: u8 = (db + 2).try_into().unwrap();
                        bytes.push(QOI_OP_DIFF | (dr << 4) | (dg << 2) | db);
                    } else if dg >= -32
                        && dg <= 31
                        && dr_dg >= -8
                        && dr_dg <= 7
                        && db_dg >= -8
                        && db_dg <= 7
                    {
                        // Write luma diff
                        bytes.push(QOI_OP_LUMA | (dg as u8 + 32));
                        bytes.push(((dr_dg as u8 + 8) << 4) | (db_dg as u8 + 8));
                    } else {
                        // Write RGB
                        bytes.push(QOI_OP_RGB);
                        bytes.push(color.r);
                        bytes.push(color.g);
                        bytes.push(color.b);
                    }
                } else {
                    // Write RGBA
                    bytes.push(QOI_OP_RGBA);
                    bytes.push(color.r);
                    bytes.push(color.g);
                    bytes.push(color.b);
                    bytes.push(color.a);
                }
            }
        }

        prev_color = color;
    }

    QOI_END_MARKER.iter().for_each(|byte| bytes.push(*byte));

    let file_stem = Path::new(&input_filename)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    fs::write(format!("{}.qoi", file_stem), bytes).unwrap();
}
