use crate::State as PublicState;
use crate::Transition as PublicTransition;
use std::collections::HashMap;

type Name = String;

type Symbol = Option<String>;

type StateIndex = u8;

type TransitionIndex = String;

type CallbackIndex = u8;

struct State {
    on_enter: CallbackIndex,
    on_exit: CallbackIndex,
    outgoing_transitions: HashMap<Symbol, Vec<TransitionIndex>>,
}

impl State {
    fn new(on_enter: CallbackIndex, on_exit: CallbackIndex) -> State {
        State {
            on_enter,
            on_exit,
            outgoing_transitions: HashMap::new(),
        }
    }
}

struct Transition {
    output: CallbackIndex,
    following_state: StateIndex,
}

pub struct StateMachine {
    states: Vec<State>,
    transitions: Vec<Transition>,
    states_by_name: HashMap<Name, StateIndex>,
    callbacks: Vec<Box<dyn FnMut() -> ()>>,

    initial_states: Vec<StateIndex>,
    current_states: Vec<StateIndex>,

    transitions_buffer: Vec<TransitionIndex>,
    next_states_buffer: Vec<StateIndex>,
}

impl StateMachine {
    pub fn step(&mut self, symb: &Symbol) {
        self.transitions_buffer.clear();
        self.next_states_buffer.clear();

        for &i_state in &self.current_states {
            let state = &mut self.states[i_state as usize];
            let on_exit = &mut self.callbacks[state.on_exit as usize];
            on_exit();
            self.transitions_buffer
                .extend(state.outgoing_transitions.get_mut(&None).unwrap().iter());
            self.transitions_buffer
                .extend(state.outgoing_transitions.get_mut(symb).unwrap().iter());
        }

        for &i_transition in &self.transitions_buffer {
            let transition = &mut self.transitions[i_transition as usize];
            let output = &mut self.callbacks[transition.output as usize];
            output();
            self.next_states_buffer.push(transition.following_state);
        }

        for &i_state in &self.next_states_buffer {
            let state = &mut self.states[i_state as usize];
            let on_enter = &mut self.callbacks[state.on_enter as usize];
            on_enter();
        }
        std::mem::swap(&mut self.current_states, &mut self.next_states_buffer);
    }

    pub fn init(&mut self, initial_states: &Vec<Name>) {
        for name in initial_states {
            self.initial_states.push(*self.states_by_name.get(name).unwrap());
        }
        self.states_by_name.clear();
        self.states_by_name.shrink_to_fit();
        self.rewind();
    }

    pub fn rewind(&mut self) {
        self.current_states = self.initial_states.clone();
    }

    pub fn add_state(&mut self, state: PublicState) {
        self.callbacks.push(state.on_enter);
        self.callbacks.push(state.on_exit);
        self.states.push(State::new(
            (self.callbacks.len() - 2) as CallbackIndex,
            (self.callbacks.len() - 1) as CallbackIndex,
        ));
        self.states_by_name
            .insert(state.name, (self.states.len() - 1) as StateIndex);
    }

    pub fn add_transition(&mut self, transition: PublicTransition) {
        self.callbacks.push(transition.output);
        self.transitions.push(Transition {
            output: (self.callbacks.len() - 1) as CallbackIndex,
            following_state: *self
                .states_by_name
                .get(&transition.following_state)
                .unwrap() as StateIndex,
        });
        self.states[*self.states_by_name.get(&transition.previous_state).unwrap() as usize]
            .outgoing_transitions
            .get_mut(&transition.symbol)
            .unwrap()
            .push((self.transitions.len() - 1) as TransitionIndex);
    }
}
