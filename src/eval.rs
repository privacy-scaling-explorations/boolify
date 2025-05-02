use std::collections::HashMap;

use bristol_circuit::BristolCircuit;

pub fn eval(circuit: &BristolCircuit, inputs: &HashMap<String, usize>) -> HashMap<String, usize> {
    let mut wires: Vec<Option<bool>> = vec![None; circuit.wire_count];

    for input in &circuit.info.inputs {
        let value = inputs.get(&input.name).expect("missing input value");

        if input.width < (usize::BITS as usize) {
            assert!(*value >> input.width == 0, "input value too large");
        }

        for j in 0..input.width {
            wires[input.address + j] = Some((value >> j) & 1 == 1);
        }
    }

    for gate in &circuit.gates {
        match gate.op.as_str() {
            "AND" => {
                let a = gate.inputs[0];
                let b = gate.inputs[1];
                let c = gate.outputs[0];

                wires[c] = Some(wires[a].unwrap() && wires[b].unwrap());
            }
            "XOR" => {
                let a = gate.inputs[0];
                let b = gate.inputs[1];
                let c = gate.outputs[0];

                wires[c] = Some(wires[a].unwrap() ^ wires[b].unwrap());
            }
            "INV" => {
                let a = gate.inputs[0];
                let c = gate.outputs[0];

                wires[c] = Some(!wires[a].unwrap());
            }
            "COPY" => {
                let a = gate.inputs[0];
                let c = gate.outputs[0];

                wires[c] = Some(wires[a].unwrap());
            }
            _ => {
                panic!("unknown gate operation: {}", gate.op);
            }
        }
    }

    let mut outputs = HashMap::<String, usize>::new();

    for output in &circuit.info.outputs {
        let mut value = 0;

        for j in 0..output.width {
            value |= (wires[output.address + j].unwrap() as usize) << j;
        }

        outputs.insert(output.name.clone(), value);
    }

    outputs
}
