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

            &CharSet::Epsilon => true,
        }
    }
}

pub struct NFA {
    cur_states: HashSet<usize>,
    transitions: HashMap<usize, Vec<(CharSet, usize)>>,
    accepting: HashSet<usize>,
}

impl NFA {
    pub fn new(transitions: HashMap<usize, Vec<(CharSet, usize)>>, accepting: HashSet<usize>) -> NFA {
        let mut nfa = NFA {
            cur_states: HashSet::new(),
            transitions: transitions,
            accepting: accepting,
        };
        nfa.reset();
        nfa
    }

    pub fn run(&mut self, mut chars: Chars) -> bool {
        loop {
            match chars.next() {
                None => {
                    return self.check_accepting();
                }
                Some(c) => {
                    self.step(c);
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.cur_states.clear();
        self.cur_states.insert(0);
        self.take_epsilons();
    }

    pub fn feed(&mut self, c: char) {
        self.step(c);
    }

    pub fn check_accepting(&self) -> bool {
        for state in self.cur_states.iter() {
            if self.accepting.contains(state) {
                return true;
            }
        }
        false
    }

    fn step(&mut self, c: char) {
        let mut new_states: HashSet<usize> = HashSet::with_capacity(self.cur_states.len());
        for cur_state in self.cur_states.iter() {
            if let Some(ts) = self.transitions.get(cur_state) {
                for &(ref cs, ref t) in ts {
                    if cs.test(c) {
                        new_states.insert(*t);
                    }
                }
            }
        }
        std::mem::swap(&mut self.cur_states, &mut new_states);

        self.take_epsilons();
    }

    fn take_epsilons(&mut self) {
        let mut new_states = HashSet::with_capacity(self.cur_states.len());

        loop {
            for cur_state in self.cur_states.iter() {
                if let Some(ts) = self.transitions.get(cur_state) {
                    for &(ref cs, ref t) in ts.iter() {
                        match cs {
                            &CharSet::Epsilon => {
                                if !self.cur_states.contains(t) {
                                    new_states.insert(*t);
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }

            if new_states.is_empty() {
                break;
            } else {
                self.cur_states.extend(&new_states);
                new_states.clear();
            }
        }
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
    pub fn build(regex: &Regex) -> NFA {
        let mut builder = NFABuilder {
            next_state: 1,
            transitions: HashMap::new(),
        };

        let accepting_states = builder.add_regex(&vec![0], regex);

        NFA::new(builder.transitions, HashSet::from_iter(accepting_states.into_iter()))
    }

    fn add_regex(&mut self, current_states: &[usize], regex: &Regex) -> Vec<usize> {
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
                let next_states = self.add_regex(current_states, r1);
                self.add_regex(&next_states, r2)
            }

            &Regex::Or(ref r1, ref r2) => {
                let mut next_states_1 = self.add_regex(current_states, r1);
                let mut next_states_2 = self.add_regex(current_states, r2);
                let mut ret = Vec::with_capacity(next_states_1.len() + next_states_2.len());
                ret.append(&mut next_states_1);
                ret.append(&mut next_states_2);
                ret
            }

            &Regex::Star(ref r) => {
                let next_states = self.add_regex(current_states, r);
                // add epsilon transitions from next states to current states
                for next_state in next_states {
                    for current_state in current_states {
                        self.add_transition(next_state, &CharSet::Epsilon, *current_state)
                    }
                }
                current_states.to_owned()
            }

            &Regex::Plus(ref r) => {
                let next_states = self.add_regex(current_states, r);
                let r_cloned: Box<Regex> = r.clone();
                self.add_regex(&next_states, &Regex::Star(r_cloned))
            }

            &Regex::Ques(ref r) => {
                let mut next_states_1 = current_states.to_owned();
                let mut next_states_2 = self.add_regex(current_states, r);
                next_states_1.append(&mut next_states_2);
                next_states_1
            }
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
    fn regex_seq() {
        let cs1 = CharSet::SingleChar('a');
        let cs2 = CharSet::SingleChar('b');
        let cs3 = CharSet::SingleChar('c');
        let r1  = Regex::Seq(
                    Box::new(Regex::CharSet(cs1)),
                    Box::new(Regex::Seq(Box::new(Regex::CharSet(cs2)),
                                        Box::new(Regex::CharSet(cs3)))));


        let mut nfa = NFABuilder::build(&r1);
        assert!(nfa.run("abc".chars()));

        nfa.reset();
        assert!(!nfa.run("".chars()));

        nfa.reset();
        assert!(!nfa.run("a".chars()));

        nfa.reset();
        assert!(!nfa.run("ab".chars()));

        nfa.reset();
        assert!(!nfa.run("abcd".chars()));
    }

    #[test]
    fn regex_or() {
        let cs1 = CharSet::SingleChar('a');
        let cs2 = CharSet::SingleChar('b');
        let cs3 = CharSet::SingleChar('c');
        let r1  = Regex::Or(
                    Box::new(Regex::CharSet(cs1)),
                    Box::new(Regex::Or(Box::new(Regex::CharSet(cs2)),
                                       Box::new(Regex::CharSet(cs3)))));

        let mut nfa = NFABuilder::build(&r1);
        assert!(nfa.run("a".chars()));

        nfa.reset();
        assert!(nfa.run("b".chars()));

        nfa.reset();
        assert!(nfa.run("c".chars()));

        nfa.reset();
        assert!(!nfa.run("ac".chars()));
    }

    #[test]
    fn regex_eps() {
        let r1  = Regex::Eps;

        let mut nfa = NFABuilder::build(&r1);
        assert!(nfa.run("".chars()));
    }

    #[test]
    fn regex_star() {
        let cs1 = CharSet::SingleChar('a');
        let r1  = Regex::Star(Box::new(Regex::CharSet(cs1)));

        let mut nfa = NFABuilder::build(&r1);
        assert!(nfa.run("".chars()));

        nfa.reset();
        assert!(nfa.run("a".chars()));

        nfa.reset();
        assert!(nfa.run("aa".chars()));

        nfa.reset();
        assert!(nfa.run("aaa".chars()));
    }

    #[test]
    fn regex_plus() {
        let cs1 = CharSet::SingleChar('a');
        let r1  = Regex::Plus(Box::new(Regex::CharSet(cs1)));

        let mut nfa = NFABuilder::build(&r1);
        assert!(!nfa.run("".chars()));

        nfa.reset();
        assert!(nfa.run("a".chars()));

        nfa.reset();
        assert!(nfa.run("aa".chars()));

        nfa.reset();
        assert!(nfa.run("aaa".chars()));
    }

    #[test]
    fn regex_ques() {
        let cs1 = CharSet::SingleChar('a');
        let r1  = Regex::Ques(Box::new(Regex::CharSet(cs1)));

        let mut nfa = NFABuilder::build(&r1);
        assert!(nfa.run("".chars()));

        nfa.reset();
        assert!(nfa.run("a".chars()));

        nfa.reset();
        assert!(!nfa.run("aa".chars()));
    }

    #[test]
    fn regex_complex() {
        let cs1 = CharSet::SingleChar('a');
        let cs2 = CharSet::SingleChar('b');
        let r1  = Regex::Or(
                    Box::new(Regex::Ques(Box::new(Regex::CharSet(cs1)))),
                    Box::new(Regex::Ques(Box::new(Regex::CharSet(cs2)))));

        let mut nfa = NFABuilder::build(&r1);
        assert!(nfa.run("".chars()));

        nfa.reset();
        assert!(nfa.run("a".chars()));

        nfa.reset();
        assert!(nfa.run("b".chars()));

        nfa.reset();
        assert!(!nfa.run("c".chars()));

        nfa.reset();
        assert!(!nfa.run("ab".chars()));
    }
}
