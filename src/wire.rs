use embedded_hal::digital::ErrorType;
use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin};
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::Mutex;

type PinId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WireState {
	Low,
	High,
	Floating,
}

impl Copy for WireState {}

#[derive(Debug)]
struct WireWrapper {
	pub state: Vec<WireState>,
	pub pull: WireState,
}

impl WireWrapper {
	fn new() -> Self {
		Self::new_with_pull(WireState::Floating)
	}

	fn new_with_pull(pull: WireState) -> Self {
		WireWrapper {
			state: vec![],
			pull,
		}
	}
}

impl Default for WireWrapper {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone, Debug)]
pub struct Wire {
	wire: Arc<Mutex<WireWrapper>>,
}

impl Wire {
	pub fn new() -> Self {
		Self::new_with_pull(WireState::Floating)
	}

	pub fn new_with_pull(pull: WireState) -> Self {
		Self {
			wire: Arc::new(Mutex::new(WireWrapper::new_with_pull(pull))),
		}
	}

	pub fn set_state(&mut self, id: PinId, state: WireState) {
		let mut wire = self.wire.lock().unwrap();
		wire.state[id] = state;
		// check for short circuit
		let _ = Self::wire_state(&wire);
	}

	pub fn get_pin_state(&self, id: PinId) -> WireState {
		self.wire.lock().unwrap().state[id]
	}

	pub fn update_pin_state<F>(&mut self, id: PinId, mut f: F)
	where
		F: FnMut(WireState) -> WireState,
	{
		let mut wire = self.wire.lock().unwrap();
		wire.state[id] = f(wire.state[id]);
		// check for short circuit
		let _ = Self::wire_state(&wire);
	}

	pub fn get_state(&self) -> WireState {
		let wire = self.wire.lock().unwrap();
		Self::wire_state(&wire)
	}

	fn wire_state(wire: &WireWrapper) -> WireState {
		use WireState::*;
		let mut s = Floating;
		for state in wire.state.iter() {
			if *state == Floating {
				continue;
			}
			if s != Floating && *state != Floating && *state != s {
				panic!("short circuit: {:?}", wire.state);
			}
			s = *state;
		}
		if s == WireState::Floating {
			wire.pull
		} else {
			s
		}
	}

	pub fn connect_push_pull_pin(&self) -> PushPullPin {
		let mut wire = self.wire.lock().unwrap();
		let id = wire.state.len();
		wire.state.push(WireState::Floating);
		PushPullPin {
			id,
			wire: self.clone(),
		}
	}

	pub fn connect_open_drain_pin(&self) -> OpenDrainPin {
		let mut wire = self.wire.lock().unwrap();
		let id = wire.state.len();
		wire.state.push(WireState::Floating);
		OpenDrainPin {
			id,
			wire: self.clone(),
		}
	}

	pub fn connect_input_pin(&self) -> InputOnlyPin {
		InputOnlyPin { wire: self.clone() }
	}
}

impl Default for Wire {
	fn default() -> Self {
		Self::new()
	}
}

pub struct InputOnlyPin {
	wire: Wire,
}

impl ErrorType for InputOnlyPin {
	type Error = Infallible;
}

impl InputPin for InputOnlyPin {
	fn is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_state() == WireState::High)
	}

	fn is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_state() == WireState::Low)
	}
}

pub struct PushPullPin {
	wire: Wire,
	id: PinId,
}

impl ErrorType for PushPullPin {
	type Error = Infallible;
}

impl InputPin for PushPullPin {
	fn is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_state() == WireState::High)
	}

	fn is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_state() == WireState::Low)
	}
}

impl OutputPin for PushPullPin {
	fn set_low(&mut self) -> Result<(), Self::Error> {
		self.wire.set_state(self.id, WireState::Low);
		Ok(())
	}

	fn set_high(&mut self) -> Result<(), Self::Error> {
		self.wire.set_state(self.id, WireState::High);
		Ok(())
	}
}

impl StatefulOutputPin for PushPullPin {
	fn is_set_high(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_pin_state(self.id) == WireState::High)
	}

	fn is_set_low(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_pin_state(self.id) == WireState::Low)
	}
}

impl ToggleableOutputPin for PushPullPin {
	fn toggle(&mut self) -> Result<(), Self::Error> {
		self.wire.update_pin_state(self.id, |x| match x {
			WireState::Low => WireState::High,
			WireState::High => WireState::Low,
			WireState::Floating => WireState::Low,
		});
		Ok(())
	}
}

pub struct OpenDrainPin {
	wire: Wire,
	id: PinId,
}

impl ErrorType for OpenDrainPin {
	type Error = Infallible;
}

impl InputPin for OpenDrainPin {
	fn is_high(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_state() == WireState::High)
	}

	fn is_low(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_state() == WireState::Low)
	}
}

impl OutputPin for OpenDrainPin {
	fn set_low(&mut self) -> Result<(), Self::Error> {
		self.wire.set_state(self.id, WireState::Floating);
		Ok(())
	}

	fn set_high(&mut self) -> Result<(), Self::Error> {
		self.wire.set_state(self.id, WireState::Low);
		Ok(())
	}
}

impl StatefulOutputPin for OpenDrainPin {
	fn is_set_high(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_pin_state(self.id) == WireState::Low)
	}

	fn is_set_low(&self) -> Result<bool, Self::Error> {
		Ok(self.wire.get_pin_state(self.id) == WireState::Floating)
	}
}

impl ToggleableOutputPin for OpenDrainPin {
	fn toggle(&mut self) -> Result<(), Self::Error> {
		self.wire.update_pin_state(self.id, |x| match x {
			WireState::Floating => WireState::Low,
			WireState::Low => WireState::Floating,
			WireState::High => WireState::Floating,
		});
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use WireState::*;

	#[test]
	fn init() {
		let wire = Wire::new();
		let wire2 = Wire::default();
		assert_eq!(wire.get_state(), wire2.get_state());
		assert_eq!(Floating, wire.get_state());
		let wire = Wire::new_with_pull(High);
		assert_eq!(High, wire.get_state());
	}

	#[test]
	fn pull_up() {
		let wire = Wire::new_with_pull(High);
		let mut pin = wire.connect_open_drain_pin();
		assert_eq!(High, wire.get_state());
		assert_eq!(Ok(()), pin.set_high());
		assert_eq!(Low, wire.get_state());
		assert_eq!(Ok(false), pin.is_set_low());
		assert_eq!(Ok(true), pin.is_set_high());
		assert_eq!(Ok(()), pin.toggle());
		assert_eq!(High, wire.get_state());
		assert_eq!(Ok(()), pin.toggle());
		assert_eq!(Low, wire.get_state());
	}

	#[test]
	fn pull_down() {
		let wire = Wire::new_with_pull(Low);
		let mut pin = wire.connect_push_pull_pin();
		assert_eq!(Low, wire.get_state());
		assert_eq!(Ok(()), pin.set_high());
		assert_eq!(Ok(false), pin.is_set_low());
		assert_eq!(Ok(true), pin.is_set_high());
		assert_eq!(High, wire.get_state());
		assert_eq!(Ok(()), pin.set_low());
		assert_eq!(Low, wire.get_state());
		assert_eq!(Ok(true), pin.is_set_low());
		assert_eq!(Ok(false), pin.is_set_high());
		assert_eq!(Ok(()), pin.toggle());
		assert_eq!(High, wire.get_state());
		assert_eq!(Ok(()), pin.toggle());
		assert_eq!(Low, wire.get_state());
	}

	#[test]
	fn input() {
		let wire = Wire::new();
		let mut pin_out = wire.connect_push_pull_pin();
		let pin_in = wire.connect_input_pin();
		assert_eq!(Floating, wire.get_state());
		assert_eq!(Ok(false), pin_in.is_high());
		assert_eq!(Ok(false), pin_in.is_low());
		assert_eq!(Ok(()), pin_out.set_low());
		assert_eq!(Low, wire.get_state());
		assert_eq!(Ok(false), pin_in.is_high());
		assert_eq!(Ok(true), pin_in.is_low());
		assert_eq!(Ok(()), pin_out.set_high());
		assert_eq!(High, wire.get_state());
		assert_eq!(Ok(true), pin_in.is_high());
		assert_eq!(Ok(false), pin_in.is_low());
	}

	#[test]
	#[should_panic]
	fn short_circuit() {
		let wire = Wire::new();
		let mut pin1 = wire.connect_push_pull_pin();
		let mut pin2 = wire.connect_push_pull_pin();
		assert_eq!(Ok(()), pin1.set_high());
		// this will cause a short circuit and panic
		assert_eq!(Ok(()), pin2.set_low());
	}

	#[test]
	fn multiple_pins() {
		let wire = Wire::new();
		let mut pin1 = wire.connect_push_pull_pin();
		let mut pin2 = wire.connect_open_drain_pin();
		let pin3 = wire.connect_input_pin();
		assert_eq!(Ok(()), pin1.set_high());
		assert_eq!(Ok(true), pin3.is_high());
		assert_eq!(Ok(()), pin2.set_low());
		assert_eq!(Ok(true), pin3.is_high());
		assert_eq!(Ok(()), pin1.set_low());
		assert_eq!(Ok(true), pin3.is_low());
		assert_eq!(Ok(()), pin2.set_high());
		assert_eq!(Ok(true), pin3.is_low());
	}
}
