use std::collections::HashMap;

use bristol_circuit::BristolCircuit;

pub fn eval(circuit: &BristolCircuit, inputs: &HashMap<String, usize>) -> HashMap<String, usize> {
    let mut wires = vec![false; circuit.wire_count];

    let mut sorted_inputs = circuit
        .info
        .input_name_to_wire_index
        .iter()
        .collect::<Vec<_>>();

    sorted_inputs.sort_by(|a, b| a.1.cmp(b.1));

    let (input_widths, output_widths) = circuit
        .io_widths
        .as_ref()
        .expect("boolean circuit requires io widths");

    assert!(sorted_inputs.len() == input_widths.len());

    for i in 0..sorted_inputs.len() {
        let (name, id_start) = sorted_inputs[i];
        let width = input_widths[i];

        let value = inputs.get(name).expect("missing input value");

        assert!(*value >> width == 0, "input value too large");

        for j in 0..width {
            wires[id_start + j] = (value >> (width - j - 1)) & 1 == 1;
        }
    }

    for gate in &circuit.gates {
        match gate.op.as_str() {
            "AND" => {
                let a = gate.inputs[0];
                let b = gate.inputs[1];
                let c = gate.outputs[0];

                wires[c] = wires[a] && wires[b];
            }
            "OR" => {
                let a = gate.inputs[0];
                let b = gate.inputs[1];
                let c = gate.outputs[0];

                wires[c] = wires[a] || wires[b];
            }
            "XOR" => {
                let a = gate.inputs[0];
                let b = gate.inputs[1];
                let c = gate.outputs[0];

                wires[c] = wires[a] ^ wires[b];
            }
            "NOT" => {
                let a = gate.inputs[0];
                let c = gate.outputs[0];

                wires[c] = !wires[a];
            }
            "COPY" => {
                let a = gate.inputs[0];
                let c = gate.outputs[0];

                wires[c] = wires[a];
            }
            _ => {
                panic!("unknown gate operation: {}", gate.op);
            }
        }
    }

    let mut outputs = HashMap::<String, usize>::new();

    let mut sorted_outputs = circuit
        .info
        .output_name_to_wire_index
        .iter()
        .collect::<Vec<_>>();

    sorted_outputs.sort_by(|a, b| a.1.cmp(b.1));

    assert!(sorted_outputs.len() == output_widths.len());

    for i in 0..sorted_outputs.len() {
        let (name, id_start) = sorted_outputs[i];
        let width = output_widths[i];

        let mut value = 0;

        for j in 0..width {
            value |= (wires[id_start + j] as usize) << (width - j - 1);
        }

        outputs.insert(name.clone(), value);
    }

    outputs
}
