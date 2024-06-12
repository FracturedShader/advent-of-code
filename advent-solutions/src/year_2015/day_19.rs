use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    io::BufRead,
    ops::Range,
};

/// Heart of the DFA.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Transition {
    /// Mask for determining if an input has a transition, and the offset from the current index to
    /// find it at.
    ///
    /// # Examples
    /// ```
    /// let input = 0b0010;
    /// let gate = Transition::Gate(0b0010);
    ///
    /// if input & gate != 0 {
    ///     let offset = 1 + gate & (input - 1);
    ///
    ///     // Add offset to the current state index to get the transition for the input
    /// } else {
    ///     // The input does not have a transition
    /// }
    /// ```
    Gate(u64),
    /// Index of a [`Transition::Gate(u64)`] to progress the DFA
    NonTerminal(usize),
    /// Index into an external set of results
    Terminal(Range<usize>),
    /// One rule ends here, but it is also a prefix for another rule
    SemiTerminal(usize, Range<usize>),
}

/// A very simple Deterministic Finite Automata (DFA) that maps sequences of ASCII letters
/// (`[a-zA-Z]+`) to an arbitrary number of replacements.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplacementDFA {
    replacements: Vec<String>,
    states: Vec<Transition>,
}

impl ReplacementDFA {
    /// Returns `(match_length, &[replacement])` for every rule that can be applied from the
    /// beginning of `segment` as a `Vec`.
    pub fn matches(&self, segment: &str) -> Vec<(usize, &[String])> {
        let mut state_idx = 0;
        let mut found = Vec::new();

        for (i, f) in segment.chars().map(Self::flag).enumerate() {
            if let Transition::Gate(mask) = self.states[state_idx] {
                if f & mask == 0 {
                    break;
                }

                let offset = ((f - 1) & mask).count_ones() + 1;

                match self
                    .states
                    .get(state_idx + offset as usize)
                    .expect("state vec should be well formed")
                {
                    Transition::NonTerminal(idx) => {
                        state_idx = *idx;
                    }
                    Transition::Terminal(range) => {
                        found.push((i + 1, &self.replacements[range.clone()]));
                        break;
                    }
                    Transition::SemiTerminal(idx, range) => {
                        state_idx = *idx;
                        found.push((i + 1, &self.replacements[range.clone()]));
                    }
                    Transition::Gate(_) => unreachable!("improperly constructed state vec"),
                }
            } else {
                unreachable!("improperly constructed state vec")
            }
        }

        found
    }

    /// Creates an iterater which produces all permutations of a string by matching and replacing
    /// successive sub-sequences using this [`ReplacementDFA`]. The iterator produces every variant
    /// that can be reached by applying exactly one rule somewhere in the provided segment.
    ///
    /// Note that a particular permutation may be seen more than once if it can be achieved via a
    /// different rule applied to a different part of the segment.
    pub fn permute<'rules, 'segment>(
        &'rules self,
        segment: &'segment str,
    ) -> DFAPermuter<'rules, 'segment> {
        DFAPermuter::new(self, segment)
    }
}

impl ReplacementDFA {
    /// Returns a merged `Vec` of all replacements strings and a `Vec<HashMap<u64, Transition>>`
    /// which represents all outgoing transitions from a particular state.
    ///
    /// This is an intermediate representation that should later be passed to
    /// [`collapse_dynamic_states`] for efficiency.
    ///
    /// # Guaranteees
    ///  - Non-terminal transitions point to another `HashMap` of transitions.
    ///  - Terminal states' `Range<usize>` map to a slice of the collapsed set of replacement
    ///  strings.
    fn build_dynamic_states(
        source: HashMap<String, Vec<String>>,
    ) -> (Vec<String>, Vec<HashMap<u64, Transition>>) {
        let mut replacements = Vec::new();
        let mut dynamic_states = vec![HashMap::<u64, Transition>::new()];

        for (mut key, values) in source {
            // Log the replacements
            let begin = replacements.len();
            replacements.extend(values);
            let end = replacements.len();

            let last = key.chars().count() - 1;
            let mut state_idx = 0;

            // Advance the state machine based on the key, correcting transitions as we go
            for c in key.drain(..last) {
                let flag = Self::flag(c);
                let mut next_idx = dynamic_states.len();

                if let Some(t) = dynamic_states[state_idx].get(&flag) {
                    let res = match t {
                        Transition::NonTerminal(idx) | Transition::SemiTerminal(idx, _) => Ok(*idx),
                        Transition::Terminal(range) => Err(range.clone()),
                        Transition::Gate(_) => unreachable!("gates don't exist yet"),
                    };

                    match res {
                        Ok(idx) => next_idx = idx,
                        Err(range) => {
                            dynamic_states[state_idx]
                                .insert(flag, Transition::SemiTerminal(next_idx, range));

                            dynamic_states.push(HashMap::new());
                        }
                    }
                } else {
                    dynamic_states[state_idx].insert(flag, Transition::NonTerminal(next_idx));

                    dynamic_states.push(HashMap::new());
                }

                state_idx = next_idx;
            }

            // Update the state machine's current state with a Terminal transition
            let flag = Self::flag(key.chars().nth(0).unwrap());

            if let Some(&Transition::NonTerminal(idx)) = dynamic_states[state_idx].get(&flag) {
                dynamic_states[state_idx].insert(flag, Transition::SemiTerminal(idx, begin..end));
            } else {
                dynamic_states[state_idx].insert(flag, Transition::Terminal(begin..end));
            }
        }

        (replacements, dynamic_states)
    }

    /// Collapses a dynamic state transition matrix into a flattened array for efficiency.
    ///
    /// # Guarantees
    ///  - Non-terminal transitions point to a `Transition::Gate(mask)` via an index.
    ///  - There are exactly `mask.count_ones()` transitions after a `Transation::Gate(mask)`.
    ///  - All transitions after a `Transition::Gate(mask)` are ordered based on flag. The
    ///  appropriate transition can be located via `state_index + ((flag - 1) & mask).count_ones()`.
    fn collapse_dynamic_states(dynamic_states: Vec<HashMap<u64, Transition>>) -> Vec<Transition> {
        // Determine offsets so every state has room for a gate plus all transitions
        let mut offset = 0;
        let state_dests = dynamic_states
            .iter()
            .map(|m| {
                let d = offset;

                offset += m.len() + 1;

                d
            })
            .collect::<Vec<_>>();

        // Flatten the flag-sorted transitions for each state and prepend the gate
        dynamic_states
            .into_iter()
            .flat_map(|m| {
                let mut flattened = m.into_iter().collect::<Vec<_>>();
                let mask = flattened.iter().fold(0, |a, &(f, _)| a | f);

                flattened.sort_by_key(|e| e.0);

                [Transition::Gate(mask)]
                    .into_iter()
                    .chain(flattened.into_iter().map(|(_, v)| match v {
                        Transition::NonTerminal(idx) => Transition::NonTerminal(state_dests[idx]),
                        Transition::SemiTerminal(idx, range) => {
                            Transition::SemiTerminal(state_dests[idx], range)
                        }
                        _ => v,
                    }))
            })
            .collect()
    }

    fn flag(c: char) -> u64 {
        match c {
            'a'..='z' => 1 << (TryInto::<u8>::try_into(c).unwrap() - b'a'),
            'A'..='Z' => 1 << (TryInto::<u8>::try_into(c).unwrap() - b'A' + 26),
            _ => unreachable!("the problem has only ASCII letters"),
        }
    }
}

impl TryFrom<HashMap<String, Vec<String>>> for ReplacementDFA {
    type Error = String;

    fn try_from(value: HashMap<String, Vec<String>>) -> Result<Self, Self::Error> {
        let (replacements, dynamic_states) = Self::build_dynamic_states(value);
        let states = Self::collapse_dynamic_states(dynamic_states);

        Ok(Self {
            replacements,
            states,
        })
    }
}

impl<S1: AsRef<str>, S2: AsRef<str>> TryFrom<&[(S1, S2)]> for ReplacementDFA {
    type Error = String;

    fn try_from(value: &[(S1, S2)]) -> Result<Self, Self::Error> {
        let mut mappings = HashMap::<String, Vec<String>>::new();

        for (k, v) in value {
            mappings
                .entry(k.as_ref().to_owned())
                .or_default()
                .push(v.as_ref().to_owned());
        }

        mappings.try_into()
    }
}

/// Produces all permutations of a string by matching and replacing successive sub-sequences using
/// a [`ReplacementDFA`]. This produces every variant that can be reached by applying exactly one
/// rule somewhere in the segment.
///
/// Note that a particular permutation may be seen more than once if it can be achieved via a
/// different rule applied to a different part of the segment.
struct DFAPermuter<'rules, 'segment> {
    rules: &'rules ReplacementDFA,

    segment: &'segment str,
    segment_len: usize,
    segment_offset: usize,

    matches: Option<Vec<(usize, &'rules [String])>>,
    match_idx: usize,
    replacement_idx: usize,
}

impl<'rules, 'segment> DFAPermuter<'rules, 'segment> {
    pub fn new(rules: &'rules ReplacementDFA, segment: &'segment str) -> Self {
        Self {
            rules,

            segment,
            segment_len: segment.len(),
            segment_offset: 0,

            matches: None,
            match_idx: 0,
            replacement_idx: 0,
        }
    }
}

impl<'rules, 'segment> Iterator for DFAPermuter<'rules, 'segment> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.segment_offset == self.segment_len {
                break None;
            } else if let Some(matches) = &self.matches {
                let (count, replacements) = matches[self.match_idx];
                let prefix = &self.segment[0..self.segment_offset];
                let suffix = &self.segment[(self.segment_offset + count)..];

                let res = format!("{prefix}{}{suffix}", replacements[self.replacement_idx]);

                self.replacement_idx += 1;

                if self.replacement_idx == replacements.len() {
                    self.replacement_idx = 0;
                    self.match_idx += 1;

                    if self.match_idx == matches.len() {
                        self.matches = None;
                        self.match_idx = 0;
                        self.segment_offset += 1;
                    }
                }

                break Some(res);
            }

            let m = self.rules.matches(&self.segment[self.segment_offset..]);

            if m.is_empty() {
                self.segment_offset += 1;
            } else {
                self.matches = Some(m);
                self.match_idx = 0;
                self.replacement_idx = 0;
            }
        }
    }
}

fn parse_puzzle_input<I>(lines: I) -> Option<(Vec<(String, String)>, String)>
where
    I: Iterator<Item = String>,
{
    let mut mappings = Vec::new();

    for line in lines {
        if line.is_empty() {
            continue;
        }

        if line.contains("=>") {
            let mut pieces = line.split(" => ");
            let key = pieces
                .next()
                .expect("replacement lines should have the format \"strA => strB\"")
                .to_owned();
            let value = pieces
                .next()
                .expect("replacement lines should have the format \"strA => strB\"")
                .to_owned();

            mappings.push((key, value));
        } else {
            return Some((mappings, line));
        }
    }

    None
}

/// Determines the minimum number of rule applications required to get from "e" to the provided
/// string. If no combination of provided rules can ever reach the target string, `None` is
/// returned. This discovery may take quite some time since all possible states will be visited.
fn min_construction_steps(rules: &[(String, String)], value: &str) -> Option<u32> {
    // We take advantage of the hard lower bound and reverse the problem. Instead of starting at
    // "e" and trying to get to `value`, we start at `value` and work backwards to "e".
    let rules = rules.iter().map(|(f, t)| (t, f)).collect::<Vec<_>>();
    let dfa: ReplacementDFA = rules
        .as_slice()
        .try_into()
        .expect("it should be possible to construct a replacement DFA from reversed rules");

    // We perform an A* search with our heuristic for potential remaining rule transformations
    // simply being the length of the string.
    let mut min_steps = HashMap::<String, u32>::new();
    let mut queue = BinaryHeap::new();

    min_steps.insert(value.to_owned(), 0);

    queue.push((std::cmp::Reverse(value.len()), value.to_owned()));

    while let Some((_, node)) = queue.pop() {
        let node_dist = *min_steps
            .get(&node)
            .expect("nodes should have a distance before being visited");

        if node == "e" {
            return Some(node_dist);
        }

        let neighbor_dist = node_dist + 1;

        queue.extend(dfa.permute(&node).filter_map(|s| {
            let neighbor_min = min_steps.entry(s.clone()).or_insert(u32::MAX);

            if neighbor_dist < *neighbor_min {
                *neighbor_min = neighbor_dist;

                Some((std::cmp::Reverse((neighbor_dist as usize) + s.len()), s))
            } else {
                None
            }
        }));
    }

    None
}

pub fn part_01(reader: Option<impl BufRead>) {
    let reader = reader.expect("data should be available for this problem");

    let (replacements, input) = parse_puzzle_input(reader.lines().map_while(Result::ok))
        .expect("input should have some replacement definitions and an input");

    let dfa: ReplacementDFA = replacements
        .as_slice()
        .try_into()
        .expect("puzzle input should fit in depth-2 DFA");

    let results: HashSet<_> = dfa.permute(&input).collect();
    let num_distinct = results.len();

    println!("Distinct molecules: {num_distinct}");
}

pub fn part_02(reader: Option<impl BufRead>) {
    let reader = reader.expect("data should be available for this problem");

    let (replacements, input) = parse_puzzle_input(reader.lines().map_while(Result::ok))
        .expect("input should have some replacement definitions and an input");

    let min_steps = min_construction_steps(&replacements, &input)
        .expect("full reduction should be possible with provided rules");

    println!("Steps to construct molecule: {min_steps}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unique_hoh() {
        let rules = [("H", "HO"), ("H", "OH"), ("O", "HH")];
        let dfa: ReplacementDFA = rules.as_slice().try_into().unwrap();

        let results: HashSet<_> = dfa.permute("HOH").collect();

        assert!(results.contains("HOOH"));
        assert!(results.contains("HOHO"));
        assert!(results.contains("OHOH"));
        assert!(results.contains("HHHH"));
        assert!(results.len() == 4);
    }

    #[test]
    fn unique_hohoho() {
        let rules = vec![("H", "HO"), ("H", "OH"), ("O", "HH")];
        let dfa: ReplacementDFA = rules.as_slice().try_into().unwrap();

        let results: HashSet<_> = dfa.permute("HOHOHO").collect();

        assert!(results.len() == 7);
    }

    #[test]
    fn min_steps_hoh() {
        let rules = [
            ("e", "H"),
            ("e", "O"),
            ("H", "HO"),
            ("H", "OH"),
            ("O", "HH"),
        ]
        .into_iter()
        .map(|(l, r)| (l.to_owned(), r.to_owned()))
        .collect::<Vec<_>>();

        let steps = min_construction_steps(&rules, "HOH");

        assert_eq!(steps.expect("reduction should succeed"), 3);
    }

    #[test]
    fn min_steps_hohoho() {
        let rules = [
            ("e", "H"),
            ("e", "O"),
            ("H", "HO"),
            ("H", "OH"),
            ("O", "HH"),
        ]
        .into_iter()
        .map(|(l, r)| (l.to_owned(), r.to_owned()))
        .collect::<Vec<_>>();

        let steps = min_construction_steps(&rules, "HOHOHO");

        assert_eq!(steps.expect("reduction should succeed"), 6);
    }
}
