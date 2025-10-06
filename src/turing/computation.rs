use super::{Alphabet, Tape, TuringMachine, LSymbol};
use std::sync::{
    Arc,
    Condvar,
    Mutex,
    atomic::{
        AtomicBool,
        AtomicUsize,
        AtomicU8,
        Ordering
    },
    mpsc::{
        self,
        Sender,
        Receiver
    }
};
use std::thread::{
    self,
};

pub struct Computation<const K: usize> {
    alphabet: Option<Alphabet>,
    tapes: Option<[Tape; K]>,
    m: Option<TuringMachine<K>>,

    current: Arc<AtomicU8>,
    transition_count: Arc<AtomicUsize>,

    w: String,

    paused:     Arc<AtomicBool>,
    stopped:    Arc<AtomicBool>,
    terminated: Arc<AtomicBool>,

    mtx:        Arc<Mutex<()>>,
    cv:         Arc<Condvar>,
    done_tx:    Option<Sender<()>>,
    done_rx:    Option<Receiver<()>>
}

#[derive(Debug)]
pub enum StepFeedback {
    CanContinue, NeedToStop
}
impl<const K: usize> Computation<K> {

    pub fn new() -> Self {

        Computation {
            alphabet: None,
            tapes: None,
            m: None,

            current: Arc::new(AtomicU8::new(0)),
            transition_count: Arc::new(AtomicUsize::new(0)),

            w: "".to_owned(),

            paused:     Arc::new(AtomicBool::new(false)),
            stopped:    Arc::new(AtomicBool::new(false)),
            terminated: Arc::new(AtomicBool::new(false)),

            mtx:        Arc::new(Mutex::new(())),
            cv:         Arc::new(Condvar::new()),

            done_rx:    None,
            done_tx:    None
        }
    }
    fn future(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.done_tx = Some(tx);
        self.done_rx = Some(rx);
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
        assert!(tape < K);
        if let Some(tapes) = &mut self.tapes {
            while let Ok(_) = tapes[tape].move_sx() {}
            for _ in 0..position {
                tapes[tape].move_dx().ok();
            }
        }
    }

    pub fn set_input_string(&mut self, w: String) -> Result<(), String> {
        if self.alphabet.is_none() { Err("An alphabet is needed".to_owned()) }
        else {
            self.w = w;
            Ok(())
        }
    }
    pub fn write_input_on_tape(&mut self) -> Result<(), String> {
        for i in 0..K { self.shift_head(0, i); }
        let alpha = self.alphabet.as_ref().ok_or("An alphabet is needed!")?;
        let tapes = self.tapes.as_mut().ok_or("Tapes are needed")?;
        for r_symbol in self.w.chars() {
            let l_symbol = alpha.get_l_symbol(&r_symbol).ok_or(format!("An error occurred between alphabet and input string, {} is not in the alphabet", r_symbol))?;
            tapes[0].write(l_symbol);
            for i in 1..K { tapes[i].write(super::BLANK); }
            for i in 0..K { tapes[i].move_dx().map_err(|_| format!("Can't move right on tape {}", i))?; }
        }
        Ok(())
    }

    pub fn output(&mut self, idx: usize) -> Result<String, String> {
        if idx >= K { return Err("Index out of bounds".to_owned()) }
        let default = self.alphabet.as_ref().ok_or("No alphabet!").unwrap().default_blank();
        let tape = self.tapes.as_mut().ok_or("No tape")?.get(idx).ok_or("No tape")?;
        let r_symbols = self.alphabet.as_ref().unwrap().get_r_symbols(tape.content());
        let mut r_tape = String::with_capacity(tape.size() + 4 + tape.head_position());
        
        for r_symbol in r_symbols {
            r_tape.push(r_symbol.unwrap_or(default));
        }
        r_tape.push_str("...\n");
        for _ in 0..tape.head_position() {
            r_tape.push(' ');
        }
        r_tape.push('^');

        Ok(r_tape)
    }
    pub fn output_all(&mut self) -> Result<String, String> {
        let tapes = self.tapes.as_mut().ok_or("No tapes")?;
        let mut string_size = 0;
        for i in 0..K {
            string_size += tapes[i].size() + 4 + tapes[i].head_position() + 1;
        }

        let mut out = String::with_capacity(
            string_size
        );
        for i in (0..K).rev() {
            out.push_str(
                self.output(i)?.as_str()
            );
            out.push('\n');
        }
        out.truncate(out.len() - 1); // Remove final \n maybe?
        Ok(out)
    }

    pub fn reset(&self) {
        {
            self.transition_count.store(0, Ordering::SeqCst);
            self.current.store(0, Ordering::SeqCst);

            self.paused.store(false, Ordering::SeqCst);
            self.stopped.store(false, Ordering::SeqCst);
            self.terminated.store(false, Ordering::SeqCst);
        }
    }

    pub fn start(c: Arc<Mutex<Self>>) -> Result<Option<Receiver<()>>, String> {
        let me = Arc::clone(&c);

        me.lock().unwrap().future();
        if !me.lock().unwrap().w.is_empty() { me.lock().unwrap().write_input_on_tape()? }

        let (paused, stopped, terminated, mtx, cv, tx, rx) = {
            let mut c = me.lock().unwrap();

            let paused: Arc<AtomicBool> = Arc::clone(&c.paused);
            let stopped = Arc::clone(&c.stopped);
            let terminated = Arc::clone(&c.terminated);
            let mtx = Arc::clone(&c.mtx);
            let cv = Arc::clone(&c.cv);
            let tx = c.done_tx.clone().ok_or("")?;
            let rx = c.done_rx.take();
        
            (paused, stopped, terminated, mtx, cv, tx, rx)
        };

        thread::spawn(move || {
            {
                let mut mec = me.lock().unwrap();
                for i in 0..K {
                    mec.shift_head(0, i);
                }
            }
            'main: loop {
                let mut guard = mtx.lock().unwrap();
                guard = cv
                    .wait_while(guard, |_| {
                        paused.load(Ordering::SeqCst)
                            && !stopped.load(Ordering::SeqCst)
                            && !terminated.load(Ordering::SeqCst)
                    }).unwrap();
                if stopped.load(Ordering::SeqCst) 
                    || terminated.load(Ordering::SeqCst) {
                    break 'main;
                }
                drop(guard);
                {
                    let mut c = me.lock().unwrap();
                    match c.step() {
                        Ok(how) => {
                            match how {
                                StepFeedback::CanContinue => { 
                                    let x = c.transition_count.load(Ordering::SeqCst);
                                    c.transition_count.store(x + 1, Ordering::SeqCst);
                                }
                                StepFeedback::NeedToStop => {
                                    c.terminated.store(true, Ordering::SeqCst);
                                    break 'main;
                                }
                            }
                        }
                        Err(_) => {
                            c.terminated.store(true, Ordering::SeqCst);
                            break 'main;
                        }
                    }
                }
            }
            let _ = tx.send(());
        });

        Ok(rx)
    }

    pub fn pause(&self) {
        if self.terminated.load(Ordering::SeqCst) || self.stopped.load(Ordering::SeqCst) {
            return
        }
        self.paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        if !self.paused.load(Ordering::SeqCst)
            || self.stopped.load(Ordering::SeqCst)
            || self.terminated.load(Ordering::SeqCst) {
            return;
        }
        {
            let _guard = self.mtx.lock().unwrap();
            self.paused.store(false, Ordering::SeqCst);
        }
        self.cv.notify_one();
    }

    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.cv.notify_all();
    }

    pub fn wait_for_termination(&self) {
        if let Some(rx) = &self.done_rx {
            let _ = rx.recv();
        }
    }

    pub fn is_on_final_state(&self) -> bool {
        if let Some(machine) = &self.m {
            let current = self.current.load(Ordering::SeqCst);
            for node in machine.final_states_reference() {
            }
            machine.is_final_state(current).unwrap_or(false)
        } else {
            false
        }
    }

    pub fn step(&mut self) -> Result<StepFeedback, String> {

        let m = self.m.as_mut().ok_or("No machine")?;
        let tapes = self.tapes.as_mut().ok_or("No tapes")?;

        let mut x: [LSymbol; K] = [0; K];
        for i in 0..K {
            x[i] = tapes[i].read();
        }
        let maybe_out = m.get_transition(self.current.load(Ordering::SeqCst), &x);
        if let Ok(out) = maybe_out {
            x = out.1;
            for i in 0..K {
                if x[i] == m.dx() {
                    tapes[i].move_dx().map_err(|_| "Could not move right".to_owned())?;
                }
                else if x[i] == m.sx() {
                    tapes[i].move_sx().map_err(|_| "Could not move left".to_owned())?;
                }
                else {
                    tapes[i].write(x[i]);
                }
            }
            self.current.store(out.0, Ordering::SeqCst);

            Ok(StepFeedback::CanContinue)
        } else {
            Ok(StepFeedback::NeedToStop)
        }
    }
}