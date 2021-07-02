use crate::grid::Rule;
use std::str::FromStr;
use regex::{Regex};

use crate::grid::Grid;

extern crate regex;

#[derive(Copy, Clone)]
pub enum Tag {
    DeadCell,
    AliveCell,
    EoL,
    EoF
}

type Pattern = (Tag, usize);
pub struct RLE {
    width: usize,
    height: usize,
    name: String,
    author: String,
    origin: (i64, i64),
    rule: String,
    patterns: Vec<Pattern>
}

const HEADER_REGEX_STRING: &str = r"^x\s?=\s?(\d+),\s?y\s?=\s?(\d+),\s?rule\s?=\s?(.+)$";
const PATTERN_REGEX_STRING: &str = r"(\d*)([bo$])";
const RULE_REGEX_STRING: &str = r"[Bb]?(\d+)\s*/\s*[Ss]?(\d+)";


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRleError {}

impl RLE {
    pub fn apply(&self, grid: &mut Grid) {
        self.set_grid(grid);
        self.set_rule(grid);
    }

    fn set_grid(&self, grid: &mut Grid) {
        let mut row: i64 = 0;
        let mut col: i64 = 0;
        for (tag, count) in self.patterns.iter() {
            for _ in 0..*count {
                match tag {
                    Tag::DeadCell => grid.set_cell(row, col, false),
                    Tag::AliveCell => grid.set_cell(row, col, true),
                    Tag::EoL => {
                        row += 1;
                        col = 0;
                        continue;
                    },
                    Tag::EoF => return,
                }

                col += 1;
            }
        }
    }

    fn set_rule(&self, grid: &mut Grid) {
        let rule_regex = Regex::new(RULE_REGEX_STRING).unwrap();
        if !rule_regex.is_match(self.rule.as_str()) {
            return
        }
        let cap = rule_regex.captures(self.rule.as_str()).unwrap();

        let mut become_alive: Vec<usize> = Vec::new();
        let mut stay_alive: Vec<usize> = Vec::new();

        let become_values = &cap[1];
        let stay_values = &cap[2];

        for c in become_values.chars() {
            let num = c.to_digit(10).unwrap() as usize;
            become_alive.push(num);
        }

        for c in stay_values.chars() {
            let num = c.to_digit(10).unwrap() as usize;
            stay_alive.push(num);
        }

        grid.set_rule(Rule {
            become_alive,
            stay_alive
        });
        
    }
}


impl FromStr for RLE {
    type Err = ParseRleError;


    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let header_regex = Regex::new(HEADER_REGEX_STRING).unwrap();
        let pattern_regex = Regex::new(PATTERN_REGEX_STRING).unwrap();

        let mut height = 0;
        let mut width = 0;
        let mut origin = (0, 0);
        let mut rule = String::new();
        let mut name = String::new();
        let mut author = String::new();

        let mut pattern_lines = String::new();
        for line in s.lines() {
            if line.starts_with("#") {
                let (nameval, authorval, originval) = parse_comment(line);
                name = nameval;
                author = authorval;
                origin = originval;
                continue;
            }
            if header_regex.is_match(line) {
                let (widthval, heightval, ruleval) = parse_header(line);
                rule = ruleval;
                height = heightval;
                width = widthval;

                continue;
            }
            
            pattern_lines.push_str(line);
        }
        
        let mut patterns: Vec<Pattern> = Vec::new();
        for cap in pattern_regex.captures_iter(&pattern_lines) {
            let count = cap.get(1).map_or(1, |c| {
                match c.as_str().parse::<usize>() {
                    Ok(v) => v,
                    Err(_) => 1,
                }
            });

            let tag: Tag = cap.get(2).map_or(Tag::EoF, |c| {
                match c.as_str() {
                    "b" => Tag::DeadCell,
                    "o" => Tag::AliveCell,
                    "$" => Tag::EoL,
                    _ => Tag::EoF
                }
            });

            patterns.push((tag, count));
        }

        Ok(RLE {
            author,
            name,
            origin,
            rule,
            height,
            width,
            patterns
        })
    }
}

fn parse_comment(s: &str) -> (String, String, (i64, i64)) {
    return (String::from(""), String::from(""), (0, 0))
}

fn parse_header(s: &str) -> (usize, usize, String) {
    let header_regex = Regex::new(HEADER_REGEX_STRING).unwrap();
    let cap = header_regex.captures(s).unwrap();
    let x = cap[1].parse::<i64>().unwrap() as usize;
    let y = cap[2].parse::<i64>().unwrap() as usize;
    let rule = match cap.get(3) {
        Some(m) => m.as_str().to_owned(),
        None => "".to_owned(),
    };
    return (x, y, rule)
}