use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
    usize,
};

use bristol_circuit::{BristolCircuit, CircuitInfo, Gate};

use crate::{
    bool_wire::{BoolData, BoolWire},
    circuit_input::CircuitInput,
    circuit_output::CircuitOutput,
};

pub fn generate_bristol(outputs: &Vec<CircuitOutput>) -> BristolCircuit {
    let mut inputs = BTreeMap::<usize, Rc<CircuitInput>>::new();

    for output in outputs {
        for bit in &output.value.bits {
            collect_inputs(&mut inputs, bit);
        }
    }

    let mut wire_id_mapper = WireIdMapper::new();

    for input in inputs.values() {
        for i in 0..input.size {
            wire_id_mapper.get(input.id_start + i);
        }
    }

    for output in outputs {
        for bit in &output.value.bits {
            if let Some(id) = bit.id() {
                wire_id_mapper.get_temp_output(id);
            }
        }
    }

    let mut gates = Vec::new();

    for output in outputs {
        for bit in &output.value.bits {
            generate_gates(&mut gates, &mut wire_id_mapper, bit);
        }
    }

    wire_id_mapper.finalize_outputs(&mut gates);

    let mut info = CircuitInfo::default();

    for input in inputs.values() {
        let id = wire_id_mapper
            .get_existing(input.id_start)
            .expect("Input should have an id");

        info.input_name_to_wire_index.insert(input.name.clone(), id);
    }

    for output in outputs {
        let first_bit = output.value.bits.first().expect("Output should have bits");

        let id = wire_id_mapper
            .get_existing(first_bit.id().expect("Output should have an id"))
            .expect("Output should have an id");

        info.output_name_to_wire_index
            .insert(output.name.clone(), id);
    }

    // TODO: represent io bit widths

    BristolCircuit {
        wire_count: wire_id_mapper.map.len(),
        info,
        gates,
    }
}

fn collect_inputs(inputs: &mut BTreeMap<usize, Rc<CircuitInput>>, bool: &BoolWire) {
    match &bool.data {
        BoolData::Input(_, input) => {
            let prev = inputs.insert(input.id_start, input.clone());

            if let Some(prev) = prev {
                assert!(std::ptr::eq(&*prev, &**input));
            }
        }
        BoolData::And(_, a, b) | BoolData::Or(_, a, b) | BoolData::Xor(_, a, b) => {
            collect_inputs(inputs, &a);
            collect_inputs(inputs, &b);
        }
        BoolData::Const(_) => (),
        BoolData::Not(_, a) => {
            collect_inputs(inputs, &a);
        }
    }
}

struct WireIdMapper {
    map: HashMap<usize, usize>,
    next_id: usize,

    temp_output_map: HashMap<usize, usize>,
    next_output_id: usize,
}

impl WireIdMapper {
    fn new() -> WireIdMapper {
        WireIdMapper {
            map: HashMap::new(),
            next_id: 0,
            temp_output_map: HashMap::new(),
            next_output_id: usize::MAX,
        }
    }

    fn get_existing(&self, old_id: usize) -> Option<usize> {
        if let Some(new_id) = self.map.get(&old_id) {
            Some(*new_id)
        } else if let Some(new_id) = self.temp_output_map.get(&old_id) {
            Some(*new_id)
        } else {
            None
        }
    }

    fn get(&mut self, old_id: usize) -> usize {
        if let Some(new_id) = self.get_existing(old_id) {
            new_id
        } else {
            let new_id = self.next_id;
            self.next_id += 1;
            self.map.insert(old_id, new_id);
            new_id
        }
    }

    fn get_temp_output(&mut self, old_id: usize) -> usize {
        if let Some(new_id) = self.get_existing(old_id) {
            new_id
        } else {
            let new_id = self.next_output_id;
            self.next_output_id -= 1;
            self.temp_output_map.insert(old_id, new_id);
            new_id
        }
    }

    fn finalize_outputs(&mut self, gates: &mut Vec<Gate>) {
        let mut update_map = HashMap::<usize, usize>::new();

        let temp_output_map_rev = self
            .temp_output_map
            .iter()
            .map(|(a, b)| (*b, *a))
            .collect::<HashMap<usize, usize>>();

        for i in (0..self.temp_output_map.len()).rev() {
            let temp_id = usize::MAX - i;
            let old_id = temp_output_map_rev
                .get(&temp_id)
                .expect("Output should exist");

            assert!(self.map.get(old_id).is_none());
            self.temp_output_map.remove(old_id);
            let proper_id = self.get(*old_id);
            update_map.insert(temp_id, proper_id);
        }

        for gate in gates {
            for input in &mut gate.inputs {
                if let Some(new_id) = update_map.get(input) {
                    *input = *new_id;
                }
            }

            for output in &mut gate.outputs {
                if let Some(new_id) = update_map.get(output) {
                    *output = *new_id;
                }
            }
        }
    }
}

fn generate_gates(gates: &mut Vec<Gate>, wire_id_mapper: &mut WireIdMapper, bit: &Rc<BoolWire>) {
    match &bit.data {
        BoolData::Input(_, _) => (),
        BoolData::And(_, a, b) | BoolData::Or(_, a, b) | BoolData::Xor(_, a, b) => {
            generate_gates(gates, wire_id_mapper, a);
            generate_gates(gates, wire_id_mapper, b);

            let a_id = wire_id_mapper.get(a.id().expect("Input should have an id"));
            let b_id = wire_id_mapper.get(b.id().expect("Input should have an id"));
            let out_id = wire_id_mapper.get(bit.id().expect("Input should have an id"));

            gates.push(Gate {
                inputs: vec![a_id, b_id],
                outputs: vec![out_id],
                op: match &bit.data {
                    BoolData::And(_, _, _) => "AND".to_string(),
                    BoolData::Or(_, _, _) => "OR".to_string(),
                    BoolData::Xor(_, _, _) => "XOR".to_string(),
                    _ => unreachable!(),
                },
            });
        }
        BoolData::Const(_) => panic!("Const should not be in the middle of the circuit"),
        BoolData::Not(_, a) => {
            generate_gates(gates, wire_id_mapper, a);

            let a = wire_id_mapper.get(a.id().expect("Input should have an id"));
            let out = wire_id_mapper.get(bit.id().expect("Input should have an id"));

            gates.push(Gate {
                inputs: vec![a],
                outputs: vec![out],
                op: "NOT".to_string(),
            });
        }
    }
}
