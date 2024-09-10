use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
};

use boolify::boolify;
use bristol_circuit::{BristolCircuit, CircuitInfo};

pub fn main() {
    let info: CircuitInfo =
        serde_json::from_str(&fs::read_to_string("input/circuit_info.json").unwrap()).unwrap();

    let circuit_file = File::open("input/circuit.txt").unwrap();

    let arith_circuit =
        BristolCircuit::read_info_and_bristol(&info, &mut BufReader::new(circuit_file)).unwrap();

    let bool_circuit = boolify(&arith_circuit, 16);

    fs::create_dir_all("output").unwrap();

    bool_circuit
        .write_bristol(&mut BufWriter::new(
            File::create("output/circuit.txt").unwrap(),
        ))
        .unwrap();

    fs::write(
        "output/circuit_info.json",
        serde_json::to_string_pretty(&bool_circuit.info).unwrap(),
    )
    .unwrap();
}
