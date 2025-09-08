use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Index;
use std::ops::Add;
use std::slice::SliceIndex;

use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;
use once_cell::sync::OnceCell;

#[derive(Debug, Clone, PartialEq)]
pub struct Grapheme {
    value: String,
}

impl Grapheme {
    pub fn new(value: &str) -> Grapheme {
        Grapheme {
            value: String::from(value),
        }
    }
}

impl std::fmt::Display for Grapheme {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GraphemeMatch<'a> {
    start: usize,
    end: usize,
    text: EzStr,
    source: &'a str,
}

impl <'a>GraphemeMatch<'a> {
    pub fn new<T, S>(start: usize, end: usize, text: T, source: S) -> Self
    where
        T: Into<EzStr>,
        S: Into<&'a str>,
    {
        GraphemeMatch { start, end, text:text.into(), source: source.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.text.data
    }

    pub fn to_ezstr(&self) -> EzStr {
        self.text.clone()
    }
}

impl Display for GraphemeMatch<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone)]
pub struct EzStr {
    pub data: String,
    pub graphemes_data: OnceCell<Vec<Grapheme>>,
    grapheme_byte_index_data: OnceCell<Vec<(usize, usize)>>, // (byte_offset, grapheme_index)
}

impl PartialEq for EzStr {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for EzStr {}

impl EzStr {
    pub fn new<S: Into<String>>(data: S) -> Self {
        let data = data.into();
        EzStr {
            data,
            graphemes_data: OnceCell::new(),
            grapheme_byte_index_data: OnceCell::new(),
        }
    }

    pub fn graphemes(&self) -> &Vec<Grapheme> {
        self.graphemes_data.get_or_init(|| {
            UnicodeSegmentation::graphemes(self.data.as_str(), true)
                .map(Grapheme::new)
                .collect()
        })
    }

    pub fn graphemes_byte_index(&self) -> &Vec<(usize, usize)> {
        self.grapheme_byte_index_data.get_or_init(|| {
            self.data
                .grapheme_indices(true)
                .enumerate()
                .map(|(gi, (bi, _))| (bi, gi))
                .collect()
        })
    }

    fn byte_range_to_grapheme_indices(&self, start: usize, end: usize) -> (usize, usize) {
        let idx = self.graphemes_byte_index();

        let g_start = match idx.binary_search_by_key(&start, |&(b, _)| b) {
            Ok(i) => idx[i].1,
            Err(i) => idx.get(i).map(|&(_, gi)| gi).unwrap_or(self.len()),
        };

        let g_end = match idx.binary_search_by_key(&end, |&(b, _)| b) {
            Ok(i) => idx[i].1,
            Err(i) => idx.get(i).map(|&(_, gi)| gi).unwrap_or(self.len()),
        };

        (g_start, g_end)
    }

    pub fn slice(&self, start: i32, end: i32) -> EzStr {
        let graphemes = self.graphemes();
        let mut ret = String::new();
        let mut start = start;
        let mut end = end;

        if start < 0 {
            start = graphemes.len() as i32 + start + 1;
        }
        if end < 0 {
            end = graphemes.len() as i32 + end + 1;
        }

        for i in start..end {
            ret += &graphemes[i as usize].value;
        }
        EzStr::new(&ret)
    }

    pub fn len(&self) -> usize {
        self.graphemes().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains<T: AsRef<str>>(&self, substring: T) -> bool {
        self.data.contains(substring.as_ref())
    }

    /// Returns the first match of the regex, in grapheme cluster indices.
    pub fn find(&self, regex: &Regex) -> Option<GraphemeMatch> {
        regex.find(&self.data).map(|m| {
            let (g_start, g_end) = self.byte_range_to_grapheme_indices(m.start(), m.end());
            GraphemeMatch::new(g_start, g_end, self.slice(g_start as i32, g_end as i32),self.data.as_str())
        })
    }

    /// Returns an iterator of matches of the regex, in grapheme cluster indices.
    pub fn find_iter<'a>(
        &'a self,
        regex: &'a Regex,
    ) -> impl Iterator<Item = GraphemeMatch> + 'a {
        regex.find_iter(&self.data).map(|m| {
            let (g_start, g_end) = self.byte_range_to_grapheme_indices(m.start(), m.end());
            GraphemeMatch::new(g_start, g_end, self.slice(g_start as i32, g_end as i32),self.data.as_str())
        })
    }
}

impl From<String> for EzStr {
    fn from(item: String) -> Self {
        EzStr::new(item)
    }
}

impl From<&str> for EzStr {
    fn from(item: &str) -> Self {
        EzStr::new(item)
    }
}

impl Into<String> for EzStr {
    fn into(self) -> String {
        self.data
    }
}



impl Index<usize> for EzStr {
    type Output = Grapheme;
    fn index(&self, index: usize) -> &Self::Output {
        &self.graphemes()[index]
    }
}

impl IntoIterator for EzStr {
    type Item = Grapheme;
    type IntoIter = std::vec::IntoIter<Grapheme>;

    fn into_iter(self) -> Self::IntoIter {
        self.graphemes().clone().into_iter()
    }
}

impl AsRef<str> for EzStr {
    fn as_ref(&self) -> &str {
        self.data.as_str()
    }
}

impl<'a> IntoIterator for &'a EzStr {
    type Item = &'a Grapheme;
    type IntoIter = std::slice::Iter<'a, Grapheme>;

    fn into_iter(self) -> Self::IntoIter {
        self.graphemes().iter()
    }
}

impl Index<std::ops::Range<usize>> for EzStr {
    type Output = [Grapheme];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.graphemes()[index]
    }
}

// EzStr + &str
impl Add<&str> for EzStr {
    type Output = EzStr;
    fn add(self, other: &str) -> EzStr {
        EzStr::new(&(self.data + other))
    }
}

// EzStr + EzStr
impl Add<&EzStr> for &EzStr {
    type Output = EzStr;
    fn add(self, other: &EzStr) -> EzStr {
        EzStr::new(&(self.data.clone() + &other.data))
    }
}

// EzStr + &str
impl Add<&str> for &EzStr {
    type Output = EzStr;
    fn add(self, other: &str) -> EzStr {
        EzStr::new(&(self.data.clone() + other))
    }
}

// EzStr + String
impl Add<&String> for &EzStr {
    type Output = EzStr;
    fn add(self, other: &String) -> EzStr {
        EzStr::new(&(self.data.clone() + other))
    }
}

impl Default for EzStr {
    fn default() -> Self {
        EzStr::new("")
    }
}

impl fmt::Display for EzStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl fmt::Debug for EzStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.data))
    }
}


