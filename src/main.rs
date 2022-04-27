use adc::*;
use dac::*;
mod adc;
mod dac;

fn main() {
    adc("siema.wav");
    dac("siema.wav");
}
