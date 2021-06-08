use crate::constants::{END_LITERAL_NUM, MAP_PREF_SIZE, MAX_OFFSET};
use std::cmp::min;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, VecDeque};

pub struct OccurrenceMap<'a> {
    data: &'a [u8],
    occur: HashMap<&'a [u8], VecDeque<usize>>,

    prefer_hidden: bool,
}

impl<'a> OccurrenceMap<'a> {
    pub fn new(data: &'a [u8], prefer_hidden: bool) -> Self {
        Self {
            data,
            occur: HashMap::new(),
            prefer_hidden,
        }
    }

    pub fn add_occurrences(&mut self, index: usize, to_advance: usize) {
        let (start_index, actual_to_advance) = if to_advance > MAX_OFFSET {
            (index + to_advance - MAX_OFFSET, MAX_OFFSET)
        } else {
            (index, to_advance)
        };

        for i in start_index..start_index + actual_to_advance {
            let pref = &self.data[i..i + MAP_PREF_SIZE];
            self.occur.entry(pref).or_default().push_back(i);
        }
    }

    pub fn get_occurrences(&mut self, index: usize) -> Occurrences {
        let pref = &self.data[index..index + MAP_PREF_SIZE];
        let entry = self.occur.entry(pref);

        match entry {
            Occupied(mut entry_val) => {
                let occur_set = entry_val.get_mut();
                occur_set.retain(|x| index - x <= MAX_OFFSET);
                debug_assert!(occur_set.len() <= u16::MAX as usize);

                if occur_set.is_empty() {
                    return Occurrences::empty();
                }
            }
            Vacant(_) => {
                return Occurrences::empty();
            }
        }

        let occur_set = &self.occur[pref];
        let match_lengths: Vec<usize> = occur_set
            .iter()
            .map(|x| calc_match_length(self.data, index, *x))
            .collect();
        if self.prefer_hidden {
            Occurrences::new(occur_set.iter().cloned().collect(), self.data, index)
        } else {
            let max_match_length = *match_lengths.iter().max().unwrap();
            let match_lengths_max: Vec<usize> = occur_set
                .iter()
                .zip(match_lengths)
                .filter(|(_val, match_length)| *match_length == max_match_length)
                .map(|(val, _match_length)| *val)
                .collect();

            Occurrences::new_with_match_length(match_lengths_max, max_match_length)
        }
    }
}

fn calc_match_length(data: &[u8], index: usize, occur_index: usize) -> usize {
    min(
        get_common_prefix_len(
            &data[index + MAP_PREF_SIZE..],
            &data[occur_index + MAP_PREF_SIZE..],
        ) + MAP_PREF_SIZE,
        data.len() - END_LITERAL_NUM - index,
    )
}

fn get_common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

pub struct Occurrences<'a> {
    data: Option<&'a [u8]>,
    occur: Vec<usize>,
    match_length: Option<usize>,
    index: usize,
}

impl<'a> Occurrences<'a> {
    fn new_with_match_length(occur: Vec<usize>, match_length: usize) -> Self {
        Self {
            data: None,
            occur,
            match_length: Some(match_length),
            index: 0,
        }
    }

    fn new(occur: Vec<usize>, data: &'a [u8], index: usize) -> Self {
        Self {
            data: Some(data),
            occur,
            match_length: None,
            index,
        }
    }

    fn empty() -> Self {
        Self {
            data: None,
            occur: Default::default(),
            match_length: Some(0),
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.occur.len()
    }

    pub fn choose_occurrence(&self, index: usize) -> (usize, usize) {
        let occur_index = self.occur[index];

        let match_length = if let Some(fixed_match_length) = self.match_length {
            fixed_match_length
        } else {
            calc_match_length(self.data.unwrap(), self.index, occur_index)
        };

        (occur_index, match_length)
    }

    pub fn get_occurrence_index(&self, chosen_index: usize) -> Option<usize> {
        self.occur.iter().position(|x| *x == chosen_index)
    }
}
