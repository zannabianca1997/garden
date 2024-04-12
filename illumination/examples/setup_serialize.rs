#![feature(duration_constants)]

use std::io::stdout;

use illumination::SunSetup;
use indent_write::io::IndentWriter;

fn main() {
    let setup = SunSetup {
        ..Default::default()
    };

    println!("Json:");
    serde_json::to_writer_pretty(IndentWriter::new("  ", stdout()), &setup).unwrap();
    println!();

    let cbor = {
        let mut buffer: Vec<u8> = vec![];
        ciborium::into_writer(&setup, &mut buffer).unwrap();
        buffer.into_boxed_slice()
    };
    let json_from_cbor: serde_json::Value = ciborium::from_reader(cbor.as_ref()).unwrap();

    println!("CBOR (to json):");
    serde_json::to_writer_pretty(IndentWriter::new("  ", stdout()), &json_from_cbor).unwrap();
    println!();
}
