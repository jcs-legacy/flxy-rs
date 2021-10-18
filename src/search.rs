/**
 * $File: search.rs $
 * $Date: 2021-10-17 19:51:03 $
 * $Revision: $
 * $Creator: Jen-Chieh Shen $
 * $Notice: See LICENSE.txt for modification and distribution information
 *                   Copyright © 2021 by Shen, Jen-Chieh $
 */
use std::collections::HashMap;

use unicode_normalization::UnicodeNormalization;

use constants::*;

/// Parsed information about a line, ready to be searched by a SearchBase.
#[derive(Debug)]
pub struct LineInfo {
    line: String,
    char_map: HashMap<char, Vec<usize>>,
    heat_map: Vec<f32>,
    factor: f32,
}

#[derive(PartialEq, Eq)]
enum CharClass {
    Separator,
    Numeric,
    Alphabetic,
    First,
    Other,
}

fn char_class(c: char) -> CharClass {
    let mut result = CharClass::Other;  // default to other
    if is_separator(c) {
        result = CharClass::Separator;
    } else if c.is_numeric() {
        result = CharClass::Numeric;
    } else if c.is_alphabetic() {
        result = CharClass::Alphabetic;
    }
    return result
}

fn is_separator(c: char) -> bool {
    return c.is_whitespace() || c.eq(&'-') || c.eq(&'_') || c.eq(&':') || c.eq(&'.') || c.eq(&'/') || c.eq(&'\\');
}

fn is_separator(c: char) -> bool {
    return c.is_whitespace() || c.eq(&'-') || c.eq(&'_') || c.eq(&':') || c.eq(&'.') || c.eq(&'/') || c.eq(&'\\');
}

pub fn score(str: &str, pattern: &str) -> Option<f32> {
    if str.is_empty() || pattern.is_empty() {
        return None;
    }
    // boost for the exact match
    let mut factor = 0.0;
    if str == pattern {
        factor = 10000.0;
    }
    let line_info = LineInfo::new(str, factor);
    let composed: Vec<char> = pattern.nfkc().filter(|ch| !ch.is_whitespace()).collect();
    line_info.score(&composed)
}

impl LineInfo {
    /// Constructs a new LineInfo objects from the given item.
    ///
    /// Factor is a "tie-breaker," or something to weight the matches in a way
    /// beyond the matching already done in flx. The greater the factor, the
    /// more greatly matching favors the item.
    pub fn new<T: Into<String>>(item: T, factor: f32) -> LineInfo {
        let mut map: HashMap<char, Vec<usize>> = HashMap::new();
        let mut heat = vec![];
        let line = item.into();

        let mut ws_score: f32 = 0.0;
        let mut cs_score: f32 = 0.0;
        let mut cur_class = CharClass::First;
        let mut cs_change = false;
        let mut last_class = CharClass::First;

        for (idx, c) in line.nfkc().enumerate() {
            if idx > MAX_LEN {
                break;
            }

            cur_class = char_class(c);

            match cur_class {
                CharClass::Separator => {
                    ws_score = SEPARATOR_FACTOR;
                },
                CharClass::First => {
                    if !c.is_whitespace() {
                        cs_score += FIRST_FACTOR;
                    }
                },
                _ => {
                    match last_class {
                        CharClass::Separator => {
                            cs_score += 10.0;
                        },
                        _ => {
                            // None..
                        },
                    };

                    if cur_class != last_class {
                        if !cs_change {
                            cs_score += CLASS_FACTOR;
                            cs_change = true;
                        }
                    } else {
                        cs_change = false;
                    }
                },
            };

            last_class = cur_class;

            map.entry(c).or_insert(Vec::default()).push(idx);
            if c.is_uppercase() {
                for lc in c.to_lowercase() {
                    map.entry(lc).or_insert(Vec::default()).push(idx);
                }
            }

            heat.push(ws_score + cs_score);

            ws_score *= SEPARATOR_REDUCE;
            if !cs_change {
                cs_score *= CLASS_REDUCE;
            }
        }

        LineInfo {
            line: line,
            char_map: map,
            heat_map: heat,
            factor: factor,
        }
    }

    /// Sets the factor for the line info
    ///
    /// Changes the factor after the creation of the line
    pub fn set_factor(&mut self, factor: f32) {
        self.factor = factor;
    }

    /// Gets the factor for the line info
    ///
    /// Produces the factor for the line info
    pub fn get_factor(&self) -> f32 {
        self.factor
    }

    fn score_position(&self, position: &[usize]) -> f32 {
        let avg_dist: f32;

        if position.len() < 2 {
            avg_dist = 0.0;
        } else {
            avg_dist = position.windows(2)
                .map(|pair| pair[1] as f32 - pair[0] as f32)
                .sum::<f32>() / position.len() as f32;
        }

        let heat_sum: f32 = position.iter()
            .map(|idx| self.heat_map[*idx])
            .sum();

        avg_dist * DIST_WEIGHT + heat_sum * HEAT_WEIGHT + self.factor * FACTOR_REDUCE
    }

    fn score<'a>(&self, query: &'a [char]) -> Option<f32> {
        let mut position = vec![0; query.len()];

        let mut lists: Vec<&[usize]> = Vec::with_capacity(query.len());

        if query.iter().any(|ch| {
            if let Some(list) = self.char_map.get(ch) {
                // Use a side effect here to save time
                lists.push(list);
                false
            } else {
                true
            }
        }) {
            return None;
        }

        self.score_inner(query, &mut position, 0, &lists)
    }

    fn score_inner<'a>(&self, query: &'a [char], position: &mut [usize], idx: usize, lists: &[&[usize]]) -> Option<f32> {
        if idx == query.len() {
            Some(self.score_position(position))
        } else {
            let mut best = None;

            for sub_position in lists[idx].iter() {
                if idx > 0 && *sub_position <= position[idx - 1] {
                    // not a valid position
                    continue;
                }

                position[idx] = *sub_position;

                if let Some(score) = self.score_inner(query, position, idx + 1, lists) {
                    if score > best.unwrap_or(::std::f32::NEG_INFINITY) {
                        best = Some(score);
                    }
                }
            }

            best
        }
    }
}
