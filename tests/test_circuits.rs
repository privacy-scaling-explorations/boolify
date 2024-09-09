use std::{cell::RefCell, rc::Rc};

use boolify::{generate_bristol, CircuitOutput, IdGenerator, ValueWire};

#[test]
fn test_2bit_add() {
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let a = ValueWire::new_input("a", 2, &id_gen);
    let b = ValueWire::new_input("b", 2, &id_gen);

    let c = ValueWire::add(&a, &b);

    let outputs = vec![CircuitOutput::new("c", c)];

    let circuit = generate_bristol(&outputs);

    let bristol_string = circuit.get_bristol_string().unwrap();

    assert_eq!(
        bristol_string,
        vec![
            "4 8",
            "2 1 1", // FIXME: Should be 2 2 2
            "1 1",   // FIXME: Should be 1 2
            "",
            "2 1 1 3 7 XOR",
            "2 1 0 2 4 XOR",
            "2 1 1 3 5 AND",
            "2 1 4 5 6 XOR",
            ""
        ]
        .join("\n")
    );
}

#[test]
fn test_2bit_mul() {
    let id_gen = Rc::new(RefCell::new(IdGenerator::new()));

    let a = ValueWire::new_input("a", 2, &id_gen);
    let b = ValueWire::new_input("b", 2, &id_gen);

    let c = ValueWire::mul(&a, &b);

    let outputs = vec![CircuitOutput::new("c", c)];

    let circuit = generate_bristol(&outputs);

    let bristol_string = circuit.get_bristol_string().unwrap();

    assert_eq!(
        bristol_string,
        vec![
            "4 8",
            "2 1 1", // FIXME: Should be 2 2 2
            "1 1",   // FIXME: Should be 1 2
            "",
            "2 1 1 3 7 AND",
            "2 1 1 2 4 AND",
            "2 1 0 3 5 AND",
            "2 1 4 5 6 XOR",
            ""
        ]
        .join("\n")
    );
}
