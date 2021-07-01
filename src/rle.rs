use std::str::FromStr;
use regex::{Error, Regex};
use lazy_static::lazy_static;

use crate::grid::Grid;

extern crate regex;
extern crate lazy_static;

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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRleError {}

impl RLE {
    pub fn set_grid(&self, grid: &mut Grid) {
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
}


impl FromStr for RLE {
    type Err = ParseRleError;


    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let header_regex = Regex::new(HEADER_REGEX_STRING).unwrap();
        let pattern_regex = Regex::new(PATTERN_REGEX_STRING).unwrap();
        let mut pattern_lines = String::new();
        for line in s.lines() {
            if line.starts_with("#") {
                let (name, author, origin) = parse_comment(line);
                continue;
            }
            if header_regex.is_match(line) {
                parse_header(line);
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
            author: String::from(""),
            name: String::from(""),
            origin: (0, 0),
            rule: String::from(""),
            height: 0,
            width: 0,
            patterns
        })
    }
}

fn parse_comment(s: &str) -> (String, String, (i64, i64)) {
    return (String::from(""), String::from(""), (0, 0))
}

fn parse_header(s: &str) -> (i64, i64, String) {
    let header_regex = Regex::new(HEADER_REGEX_STRING).unwrap();
    let cap = header_regex.captures(s).unwrap();
    let x = cap[1].parse::<i64>().unwrap();
    let y = cap[2].parse::<i64>().unwrap();
    let rule = cap[3].to_string();
    return (x, y, rule)
}