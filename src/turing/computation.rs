use super::{Alphabet, Tape, TuringMachine, State, LSymbol};

pub struct Computation<const K: usize> {
    alphabet: Option<Alphabet>,
    tapes: Option<[Tape; K]>,
    m: Option<TuringMachine<K>>,

    current: State,
    transition_count: usize,

    w: String
}

impl<const K: usize> Computation<K> {

    pub fn new() -> Self {
        Computation {
            alphabet: None,
            tapes: None,
            m: None,

            current: 0,
            transition_count: 0,

            w: "".to_owned()
        }
    }

    pub fn use_alphabet(&mut self, alpha: Alphabet) {
        self.alphabet = Some(alpha);
    }
    pub fn use_machine(&mut self, m: TuringMachine<K>) {
        self.m = Some(m);
    }
    pub fn use_tape(&mut self, t: Tape, idx: usize) {
        assert!(idx < K);
        if let Some(tapes) = &mut self.tapes {
            tapes[idx] = t;
        }
    }
    pub fn use_tapes(&mut self, tapes: [Tape; K]) {
        self.tapes = Some(tapes);
    }

    pub fn shift_head(&mut self, position: usize, tape: usize) {
        assert!(tape >= 0 && tape < K);
        if let Some(tapes) = &mut self.tapes {
            while let Ok(_) = tapes[tape].move_sx() {}
            for _ in 0..position {
                tapes[tape].move_dx().ok();
            }
        }
    }

    pub fn set_input_string(&mut self, w: String) {
        self.w = w;
    }

    pub fn start(&mut self) {
        for i in 0..K {
            self.shift_head(0, i);
        }
        self.transition_count = 0;

        while let Ok(_) = self.step() { self.transition_count += 1 }
    }

    pub fn has_accepted(&self) -> bool {
        if let Some(machine) = &self.m {
            machine.is_final_state(self.current).unwrap_or(false)
        } else {
            false
        }
    }

    pub fn step(&mut self) -> Result<(), String> {

        let m = self.m.as_mut().ok_or("No machine")?;
        let tapes = self.tapes.as_mut().ok_or("No tapes")?;

        let mut x: [LSymbol; K] = [0; K];
        for i in 0..K {
            x[i] = tapes[i].read();
        }
        println!("Current state {}, read {:?}", self.current , &x);
        let out = m.get_transition(self.current, &x).map_err(|_| String::from("Error during transition retrieval"))?;
        x = out.1;
        for i in 0..K {
            if x[i] == m.dx() {
                tapes[i].move_dx().map_err(|_| "Could not move right".to_owned())?;
                println!("SX");
            }
            else if x[i] == m.sx() {
                tapes[i].move_sx().map_err(|_| "Could not move left".to_owned())?;
                println!("DX");
            }
            else {
                tapes[i].write(x[i]);
                println!("Written {}", x[i]);
            }
        }
        self.current = out.0;

        println!("Transition done");
        Ok(())
    }
}