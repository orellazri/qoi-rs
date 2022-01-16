use clap::{ArgEnum, Parser};
use qoi_rs::{decode, encode};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(arg_enum)]
    action: Action,
    input_filename: String,
    width: u32,
    height: u32,
    channels: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum Action {
    Encode,
    Decode,
}

fn main() {
    let args = Args::parse();
    let action = args.action;
    let input_filename = args.input_filename;
    let width: u32 = args.width;
    let height: u32 = args.height;
    let channels: u8 = args.channels;
    let colorspace: u8 = 1;

    match action {
        Action::Encode => encode(&input_filename, width, height, channels, colorspace),
        Action::Decode => decode(&input_filename, width, height, channels, colorspace),
    }
}
