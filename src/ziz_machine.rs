use std::collections::HashMap;

type Symbol = Option<String>;

#[derive(Copy, Clone)]
pub struct StateIndex(pub u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TransitionIndex(pub u32);

#[derive(Copy, Clone)]
pub struct SymbolIndex(pub u32);

type Callback = Box<dyn FnMut() -> ()>;

const EMPTY_SYMBOL_INDEX: SymbolIndex = SymbolIndex(0);

pub struct StateMachine {
    states_by_name: HashMap<String, StateIndex>,
    symbols: HashMap<Symbol, SymbolIndex>,

    // Lock-step arrays, containing at position StateIndex the state callbacks.
    state_on_enter: Vec<Callback>,
    state_on_exit: Vec<Callback>,

    // Flattened 2-D array of STL[StateIndex * N_SYMBOLS + SymbolIndex] -> first Transition
    // Each pair of state and symbol gives the index of the first transtion (in
    // the transition_* arrays). The list of transitions continues until the
    // transition index of the next pair at STL[StateIndex * N_SYMBOLS + SymbolIndex + 1].
    state_transition_lists: Vec<TransitionIndex>,

    // `transition_*` arrays are in lockstep, containing the output and
    // following state for a given transition. The order they appear in these
    // arrays is:
    //     STATE_0_SYMBOL_0_TRANSITION_0
    //     STATE_0_SYMBOL_0_TRANSITION_1
    //     STATE_0_SYMBOL_0_TRANSITION_2
    //     STATE_0_SYMBOL_1_TRANSITION_0
    //     STATE_0_SYMBOL_2_TRANSITION_0
    //     STATE_0_SYMBOL_4_TRANSITION_0
    //     STATE_1_SYMBOL_0_TRANSITION_0
    //     STATE_1_SYMBOL_0_TRANSITION_1
    //     STATE_1_SYMBOL_1_TRANSITION_0
    //     STATE_1_SYMBOL_2_TRANSITION_0
    //     STATE_1_SYMBOL_3_TRANSITION_1
    //     ...
    transition_output: Vec<Callback>,
    transition_following_state: Vec<StateIndex>,

    initial_states: Vec<StateIndex>,
    current_states: Vec<StateIndex>,

    transition_range_buffer: Vec<(TransitionIndex, TransitionIndex)>,
    next_states_buffer: Vec<StateIndex>,
}

impl StateMachine {
    fn transition_range(
        &self,
        i_state: StateIndex,
        i_symbol: SymbolIndex,
    ) -> (TransitionIndex, TransitionIndex) {
        let transition_list_index = i_state.0 as usize * self.symbols.len() + i_symbol.0 as usize;
        let transition_start = self.state_transition_lists[transition_list_index];
        let transition_end = self
            .state_transition_lists
            .get(transition_list_index + 1)
            .copied()
            .unwrap_or_else(|| TransitionIndex(self.transition_output.len() as u32));
        (transition_start, transition_end)
    }

    pub fn step(&mut self, symb: Symbol) {
        self.transition_range_buffer.clear();
        self.next_states_buffer.clear();

        // Perform a single hashmap lookup for the symbol.
        let i_symbol = *self.symbols.get(&symb).unwrap();
        for &i_state in &self.current_states {
            (&mut self.state_on_exit[i_state.0 as usize])();

            // Only push (start, end) ranges, rather than all the individual transition onto the buffer.
            let empty_symbol_range = self.transition_range(i_state, EMPTY_SYMBOL_INDEX);
            if empty_symbol_range.0 != empty_symbol_range.1 {
                self.transition_range_buffer.push(empty_symbol_range);
            }
            let symbol_range = self.transition_range(i_state, i_symbol);
            if symbol_range.0 != symbol_range.1 {
                self.transition_range_buffer.push(symbol_range);
            }
        }

        // Maximise cache-locality by iterating through each array in turn, rather than in-step.
        for &(transition_start, transition_end) in &self.transition_range_buffer {
            for output in
                &mut self.transition_output[transition_start.0 as usize..transition_end.0 as usize]
            {
                output();
            }
        }

        for &(transition_start, transition_end) in &self.transition_range_buffer {
            self.next_states_buffer.extend(
                &self.transition_following_state
                    [transition_start.0 as usize..transition_end.0 as usize],
            );
        }

        for &i_state in &self.next_states_buffer {
            (&mut self.state_on_enter[i_state.0 as usize])()
        }

        std::mem::swap(&mut self.current_states, &mut self.next_states_buffer);
    }

    pub fn init(&mut self) {
        self.current_states = self.initial_states.clone();
    }
}
