use crate::value_wire::ValueWire;

#[derive(Clone)]
pub struct CircuitOutput {
    pub name: String,
    pub value: ValueWire,
}

impl CircuitOutput {
    pub fn new(name: &str, value: ValueWire) -> CircuitOutput {
        CircuitOutput {
            name: name.to_string(),
            value,
        }
    }
}
