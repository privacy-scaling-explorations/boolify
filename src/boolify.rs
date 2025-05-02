use core::panic;
use std::{collections::HashSet, rc::Rc};

use bristol_circuit::{BristolCircuit, CircuitInfo};

use crate::{generate_bristol, BoolWire, CircuitOutput, IdGenerator, ValueWire};

pub fn boolify(arith_circuit: &BristolCircuit, bit_width: usize) -> BristolCircuit {
    if !io_widths_all_1s(&arith_circuit.info) {
        panic!("Arithmetic circuit widths should all be 1s");
    }

    let id_gen = IdGenerator::new_rc_refcell();
    let mut wires: Vec<Option<ValueWire>> = vec![None; arith_circuit.wire_count];

    let mut ordered_inputs = arith_circuit.info.inputs.clone();

    // It's important to create the ValueWires in this order so that the resulting boolean circuit
    // preserves the order of the inputs.
    // (Inputs are generally already ordered this way, but we sort them just in case.)
    ordered_inputs.sort_by_key(|input| input.address);

    for input in ordered_inputs {
        wires[input.address] = Some(ValueWire::new_input(
            input.name.as_str(),
            if input.type_ == "number" {
                bit_width
            } else if input.type_ == "bool" {
                1
            } else {
                panic!("Unsupported input type: {}", input.type_)
            },
            &id_gen,
        ));
    }

    for const_info in &arith_circuit.info.constants {
        if let Some(v) = const_info.value.as_u64() {
            wires[const_info.address] =
                Some(ValueWire::new_const(v as usize, &id_gen).resize(bit_width));
        }

        if let Some(v) = const_info.value.as_bool() {
            wires[const_info.address] =
                Some(ValueWire::new_const(if v { 1 } else { 0 }, &id_gen).resize(1));
        }

        panic!("Unsupported type: {}", const_info.value);
    }

    let unary_ops = ["AUnaryAdd", "AUnarySub", "ANot", "ABitNot"]
        .iter()
        .map(|s| s.to_string())
        .collect::<HashSet<_>>();

    let binary_ops = [
        "AAdd", "ASub", "AMul", "ADiv", "AMod", "AExp", "AEq", "ANeq", "AEq", "ANeq", "ABoolAnd",
        "ABoolOr", "ALt", "ALEq", "AGt", "AGEq", "ABitAnd", "ABitOr", "AXor", "AShiftL", "AShiftR",
        "AShiftR",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect::<HashSet<_>>();

    let bool_to_value = |b: &Rc<BoolWire>| BoolWire::as_value(b).resize(1);

    for gate in &arith_circuit.gates {
        if unary_ops.contains(&gate.op) {
            assert_eq!(gate.inputs.len(), 1);
            assert_eq!(gate.outputs.len(), 1);

            let in_ = wires[gate.inputs[0]]
                .as_ref()
                .expect("Required wire not assigned");

            let out_id = gate.outputs[0];

            wires[out_id] = Some(match gate.op.as_str() {
                "AUnaryAdd" => in_.clone(),
                "AUnarySub" => ValueWire::negate(in_),
                "ANot" => bool_to_value(&BoolWire::inv(&in_.to_bool())),
                "ABitNot" => ValueWire::bit_not(in_),
                _ => unreachable!(),
            });
        } else if binary_ops.contains(&gate.op) {
            assert_eq!(gate.inputs.len(), 2);
            assert_eq!(gate.outputs.len(), 1);

            let a = wires[gate.inputs[0]]
                .as_ref()
                .expect("Required wire not assigned");

            let b = wires[gate.inputs[1]]
                .as_ref()
                .expect("Required wire not assigned");

            let out_id = gate.outputs[0];

            wires[out_id] = Some(match gate.op.as_str() {
                "AAdd" => ValueWire::add(a, b),
                "ASub" => ValueWire::sub(a, b),
                "AMul" => ValueWire::mul(a, b),
                "ADiv" => ValueWire::div(a, b),
                "AMod" => ValueWire::mod_(a, b),
                "AExp" => ValueWire::exp(a, b),
                "AEq" => bool_to_value(&ValueWire::equal(a, b)),
                "ANeq" => bool_to_value(&ValueWire::not_equal(a, b)),
                "ABoolAnd" => bool_to_value(&ValueWire::bool_and(a, b)),
                "ABoolOr" => bool_to_value(&ValueWire::bool_or(a, b)),
                "ALt" => bool_to_value(&ValueWire::less_than(a, b)),
                "ALEq" => bool_to_value(&ValueWire::less_than_or_eq(a, b)),
                "AGt" => bool_to_value(&ValueWire::greater_than(a, b)),
                "AGEq" => bool_to_value(&ValueWire::greater_than_or_eq(a, b)),
                "ABitAnd" => ValueWire::bit_and(a, b),
                "ABitOr" => ValueWire::bit_or(a, b),
                "AXor" => ValueWire::bit_xor(a, b),
                "AShiftL" => ValueWire::bit_shl(a, b),
                "AShiftR" => ValueWire::bit_shr(a, b),
                _ => unreachable!(),
            });
        } else {
            panic!("Unrecognized op: {}", &gate.op)
        }
    }

    let mut outputs = Vec::<CircuitOutput>::new();

    for output in &arith_circuit.info.outputs {
        outputs.push(CircuitOutput {
            name: output.name.clone(),
            value: wires[output.address]
                .clone()
                .expect("Required wire not assigned"),
        });
    }

    let circuit = generate_bristol(&outputs);

    drop(outputs);

    // Reverse the wires so that the parents are dropped before children. This prevents recursive
    // drop calls from overflowing the stack.
    wires.reverse();
    drop(wires);

    circuit
}

fn io_widths_all_1s(info: &CircuitInfo) -> bool {
    info.inputs
        .iter()
        .chain(info.outputs.iter())
        .all(|io| io.width == 1)
}
