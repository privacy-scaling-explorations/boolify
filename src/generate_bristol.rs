use std::{
    collections::{BTreeMap, HashMap, HashSet},
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
    let mut visited = HashSet::<usize>::new();

    for output in outputs {
        for bit in &output.value.bits {
            collect_inputs(&mut inputs, &mut visited, bit);
        }
    }

    let mut wire_id_mapper = WireIdMapper::new();

    for input in inputs.values() {
        for i in 0..input.size {
            wire_id_mapper.get(input.id_start + i);
        }
    }

    let first_input = inputs.first_key_value().expect("error: no inputs").1;
    let id_gen = &outputs.first().expect("error: no outputs").value.id_gen;

    let first_wire = Rc::new(BoolWire {
        id_gen: id_gen.clone(),
        data: BoolData::Input(first_input.id_start, first_input.clone()),
    });

    // These exist for the slightly unusual scenario where the outputs include constants -
    // we replace with these to get the required values without having to deal with any explicit
    // constants in boolean circuits, which don't usually require them
    let special_false = BoolWire::xor(&first_wire, &first_wire);
    let special_true = BoolWire::not(&special_false);

    let mut outputs = outputs.clone();
    for output in outputs.iter_mut() {
        for bit in output.value.bits.iter_mut() {
            let const_value: Option<bool> = match &bit.data {
                BoolData::Const(false) => Some(false),
                BoolData::Const(true) => Some(true),
                _ => None,
            };

            if let Some(const_value) = const_value {
                *bit = match const_value {
                    true => special_true.clone(),
                    false => special_false.clone(),
                };
            }

            let mut id = bit.id().unwrap();

            if wire_id_mapper.get_existing(id).is_some() {
                // This output wire overlaps with input!
                // That causes issues with putting output wires at the end of the circuit, so we
                // create a copy instead
                *bit = BoolWire::copy(&bit);
                id = bit.id().expect("Expected copy to produce id");
            }

            wire_id_mapper.get_temp_output(id);
        }
    }

    let mut gates = Vec::<Gate>::new();
    let mut generated_ids = HashSet::<usize>::new();

    for output in &outputs {
        for bit in &output.value.bits {
            generate_gates(&mut gates, &mut wire_id_mapper, &mut generated_ids, bit);
        }
    }

    println!("0");

    wire_id_mapper.finalize_outputs(&mut gates);

    let mut info = CircuitInfo::default();

    for input in inputs.values() {
        let id = wire_id_mapper
            .get_existing(input.id_start)
            .expect("Input should have an id");

        info.input_name_to_wire_index.insert(input.name.clone(), id);
    }

    println!("1");

    for output in &outputs {
        let first = output.value.bits.first().expect("Output should have bits");

        let id = wire_id_mapper
            .get_existing(first.id().expect("Output should have an id"))
            .expect("Output should have an id");

        info.output_name_to_wire_index
            .insert(output.name.clone(), id);
    }

    println!("2");

    let input_widths = inputs
        .values()
        .map(|input| input.size)
        .collect::<Vec<usize>>();

    let output_widths = outputs
        .iter()
        .map(|output| output.value.bits.len())
        .collect::<Vec<usize>>();

    println!("3");

    BristolCircuit {
        wire_count: wire_id_mapper.map.len(),
        info,
        io_widths: (input_widths, output_widths),
        gates,
    }
}

fn collect_inputs(
    inputs: &mut BTreeMap<usize, Rc<CircuitInput>>,
    visited: &mut HashSet<usize>,
    bool: &BoolWire,
) {
    let Some(id) = bool.id() else {
        return;
    };

    if !visited.insert(id) {
        return;
    }

    match &bool.data {
        BoolData::Input(_, input) => {
            let prev = inputs.insert(input.id_start, input.clone());

            if let Some(prev) = prev {
                assert!(std::ptr::eq(&*prev, &**input));
            }
        }
        BoolData::And(_, a, b) | BoolData::Or(_, a, b) | BoolData::Xor(_, a, b) => {
            collect_inputs(inputs, visited, &a);
            collect_inputs(inputs, visited, &b);
        }
        BoolData::Const(_) => (),
        BoolData::Not(_, a) => {
            collect_inputs(inputs, visited, &a);
        }
        BoolData::Copy(_, a) => {
            collect_inputs(inputs, visited, &a);
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

        for i in 0..self.temp_output_map.len() {
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

fn generate_gates(
    gates: &mut Vec<Gate>,
    wire_id_mapper: &mut WireIdMapper,
    generated_ids: &mut HashSet<usize>,
    start: &Rc<BoolWire>,
) {
    // The stack holds tuples of (node, visited_flag).
    // visited_flag == false: children not yet processed.
    // visited_flag == true: ready to process the node.
    let mut stack: Vec<(Rc<BoolWire>, bool)> = Vec::new();

    // If the starting node has an id and hasn't been processed yet, push it.
    if let Some(id) = start.id() {
        if generated_ids.insert(id) {
            stack.push((start.clone(), false));
        }
    }

    while let Some((bit, visited)) = stack.pop() {
        if visited {
            // Process the node after its children have been processed.
            match &bit.data {
                BoolData::Input(_, _) => { /* nothing to do for inputs */ }
                BoolData::And(_, a, b) | BoolData::Or(_, a, b) | BoolData::Xor(_, a, b) => {
                    let a_id = wire_id_mapper.get(a.id().expect("Input should have an id"));
                    let b_id = wire_id_mapper.get(b.id().expect("Input should have an id"));
                    let out_id = wire_id_mapper.get(bit.id().expect("Input should have an id"));
                    let op = match &bit.data {
                        BoolData::And(_, _, _) => "AND".to_string(),
                        BoolData::Or(_, _, _) => "OR".to_string(),
                        BoolData::Xor(_, _, _) => "XOR".to_string(),
                        _ => unreachable!(),
                    };
                    gates.push(Gate {
                        inputs: vec![a_id, b_id],
                        outputs: vec![out_id],
                        op,
                    });
                }
                BoolData::Not(_, a) | BoolData::Copy(_, a) => {
                    let a_id = wire_id_mapper.get(a.id().expect("Input should have an id"));
                    let out_id = wire_id_mapper.get(bit.id().expect("Input should have an id"));
                    let op = match &bit.data {
                        BoolData::Not(_, _) => "NOT".to_string(),
                        BoolData::Copy(_, _) => "COPY".to_string(),
                        _ => unreachable!(),
                    };
                    gates.push(Gate {
                        inputs: vec![a_id],
                        outputs: vec![out_id],
                        op,
                    });
                }
                BoolData::Const(_) => {
                    panic!("Const should not be in the middle of the circuit")
                }
            }
        } else {
            // First time seeing this node:
            // Push the node back marked as visited, then push its children.
            stack.push((bit.clone(), true));
            match &bit.data {
                BoolData::Input(_, _) => { /* no children */ }
                BoolData::And(_, a, b) | BoolData::Or(_, a, b) | BoolData::Xor(_, a, b) => {
                    // Push b then a (so that a is processed first).
                    if let Some(b_id) = b.id() {
                        if generated_ids.insert(b_id) {
                            stack.push((b.clone(), false));
                        }
                    }
                    if let Some(a_id) = a.id() {
                        if generated_ids.insert(a_id) {
                            stack.push((a.clone(), false));
                        }
                    }
                }
                BoolData::Not(_, a) | BoolData::Copy(_, a) => {
                    if let Some(a_id) = a.id() {
                        if generated_ids.insert(a_id) {
                            stack.push((a.clone(), false));
                        }
                    }
                }
                BoolData::Const(_) => {
                    panic!("Const should not be in the middle of the circuit")
                }
            }
        }
    }
}
