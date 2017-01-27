use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::str::Chars;

#[derive(Clone)]
pub enum CharSet {
    SingleChar(char),

    Range { lo: char, hi: char },

    AnyChar,

    Diff {
        include: Box<CharSet>,
        exclude: Box<CharSet>,
    },

    Union(Vec<Box<CharSet>>),

    Epsilon,
}

impl CharSet {
    pub fn test(&self, c: char) -> bool {
        match self {

            &CharSet::SingleChar(c1) => c1 == c,

            &CharSet::Range { lo, hi } => c >= lo && c <= hi,

            &CharSet::AnyChar => true,

            &CharSet::Diff { ref include, ref exclude } => include.test(c) && !exclude.test(c),

            &CharSet::Union(ref css) => {
                for cs in css {
                    if cs.test(c) {
                        return true;
                    }
                }
                false
            }

            &CharSet::Epsilon => false,
        }
    }
}

pub struct NFA {
    cur_states: Vec<usize>,
    transitions: HashMap<usize, Vec<(CharSet, usize)>>,
    accepting: HashSet<usize>,
}

impl NFA {
    pub fn run(&mut self, mut chars: Chars) -> bool {
        loop {
            let accepting = self.check_accepting();
            if accepting {
                return accepting;
            }

            match chars.next() {
                None => {
                    return accepting;
                }
                Some(c) => {
                    let mut new_states: Vec<usize> = Vec::with_capacity(self.cur_states.len());
                    for cur_state in self.cur_states.iter().cloned() {
                        match self.transitions.get(&cur_state) {
                            None => { return false; },
                            Some(ts) => {
                                for &(ref cs, ref t) in ts {
                                    if cs.test(c) {
                                        new_states.push(*t);
                                    }
                                }
                            }
                        }
                    }
                    std::mem::swap(&mut self.cur_states, &mut new_states);
                }
            }
        }
    }

    fn check_accepting(&self) -> bool {
        for state in self.cur_states.iter() {
            if self.accepting.contains(state) {
                return true;
            }
        }
        false
    }
}

#[derive(Clone)]
pub enum Regex {
    Eps,
    CharSet(CharSet),
    Seq(Box<Regex>, Box<Regex>),
    Or(Box<Regex>, Box<Regex>),
    Star(Box<Regex>),
    Plus(Box<Regex>),
    Ques(Box<Regex>),
}


pub struct NFABuilder {
    next_state: usize,
    transitions: HashMap<usize, Vec<(CharSet, usize)>>,
}

impl NFABuilder {
    pub fn new() -> NFABuilder {
        NFABuilder {
            next_state: 1,
            transitions: HashMap::new(),
        }
    }

    pub fn build(&mut self, current_states: &[usize], regex: &Regex) -> Vec<usize> {
        match regex {

            &Regex::Eps => {
                current_states.to_owned()
            }

            &Regex::CharSet(ref cs) => {
                let mut next_states = Vec::with_capacity(current_states.len());
                for state in current_states.iter().cloned() {
                    let next_state = self.new_state();
                    self.add_transition(state, cs, next_state);
                    next_states.push(next_state);
                }
                next_states
            }

            &Regex::Seq(ref r1, ref r2) => {
                let next_states = self.build(current_states, r1);
                self.build(&next_states, r2)
            }

            &Regex::Or(ref r1, ref r2) => {
                let mut next_states_1 = self.build(current_states, r1);
                let mut next_states_2 = self.build(current_states, r2);
                let mut ret = Vec::with_capacity(next_states_1.len() + next_states_2.len());
                ret.append(&mut next_states_1);
                ret.append(&mut next_states_2);
                ret
            }

            &Regex::Star(ref r) => {
                let next_states = self.build(current_states, r);
                // add epsilon transitions from next states to current states
                for next_state in next_states {
                    for current_state in current_states {
                        self.add_transition(next_state, &CharSet::Epsilon, *current_state)
                    }
                }
                current_states.to_owned()
            }

            &Regex::Plus(ref r) => {
                let next_states = self.build(current_states, r);
                let r_cloned: Box<Regex> = r.clone();
                self.build(&next_states, &Regex::Star(r_cloned))
            }

            &Regex::Ques(ref r) => {
                let mut next_states_1 = current_states.to_owned();
                let mut next_states_2 = self.build(current_states, r);
                next_states_1.append(&mut next_states_2);
                next_states_1
            }
        }
    }

    pub fn finish(self, accepting_states: Vec<usize>) -> NFA {
        NFA {
            cur_states: vec![0],
            transitions: self.transitions,
            accepting: HashSet::from_iter(accepting_states.into_iter()),
        }
    }

    fn new_state(&mut self) -> usize {
        let ret = self.next_state;
        self.next_state += 1;
        ret
    }

    fn add_transition(&mut self, from: usize, cs: &CharSet, to: usize) {
        match self.transitions.entry(from) {
            Entry::Occupied(mut ent) => {
                ent.get_mut().push((cs.clone(), to));
            },
            Entry::Vacant(ent) => {
                ent.insert(vec![(cs.clone(), to)]);
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let cs1 = CharSet::SingleChar('a');
        let cs2 = CharSet::SingleChar('b');
        let cs3 = CharSet::SingleChar('c');
        let r1  = Regex::Seq(
                    Box::new(Regex::CharSet(cs1)),
                    Box::new(Regex::Seq(Box::new(Regex::CharSet(cs2)),
                                        Box::new(Regex::CharSet(cs3)))));


        let mut nfa_builder = NFABuilder::new();
        let accepting = nfa_builder.build(&vec![0], &r1);
        let mut nfa = nfa_builder.finish(accepting);

        assert!(nfa.run("abc".chars()));
    }
}
