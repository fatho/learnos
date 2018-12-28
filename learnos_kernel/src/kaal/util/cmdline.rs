//! Kernel command line parser

/// Zero-copy command line parser
pub struct CmdLine<'a> {
    raw: &'a str,
}

impl<'a> CmdLine<'a> {
    /// Create an iterator for parsing a command line string.
    pub fn parse(raw: &str) -> CmdLine {
        CmdLine {
            raw: raw,
        }
    }

    /// Skip consecutive whitespace from the front.
    fn skip_whitespace(&mut self) {
        self.raw = self.raw.trim_start();
    }

    /// Consume the next symbol in the command line. A symbol is either
    /// - a consecutive sequence of characters that are neither whitespace nor '='
    ///   (depending on the `accept_equals` flag)
    /// - a quoted string of arbitrary characters except quotes
    fn parse_symbol(&mut self, accept_equals: bool) -> Option<&'a str> {
        let first_ch = self.raw.chars().next()?;
        // determine quote
        let is_quoted = first_ch == '"';
        
        if is_quoted {
            let rest = &self.raw[1..];
            match rest.find('"') {
                None => {
                    self.raw = "";
                    Some(rest)
                },
                Some(pos) => {
                    self.raw = &rest[pos + 1..];
                    Some(&rest[0..pos])
                }
            }
        } else {
            let s = self.raw;
            match s.find(|c: char| (!accept_equals && c == '=') || c.is_whitespace()) {
                None => {
                    self.raw = "";
                    Some(s)
                },
                Some(pos) => {
                    let (value, rest) = s.split_at(pos);
                    self.raw = rest;
                    Some(value)
                }
            }
        }
    }

    /// Parse an equal sign. Returns true if an equals sign was found, otherwise false.
    fn parse_equals(&mut self) -> bool {
        match self.raw.chars().next() {
            Some('=') => {
                self.raw = &self.raw[1..];
                true
            },
            _ => false,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CmdLineToken<'a> {
    Flag(&'a str),
    KeyValuePair(&'a str, &'a str)
}

impl<'a> Iterator for CmdLine<'a> {
    type Item = CmdLineToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        let key = self.parse_symbol(false)?;
        if self.parse_equals() {
            let value = self.parse_symbol(true).unwrap_or("");
            Some(CmdLineToken::KeyValuePair(key, value))
        } else {
            Some(CmdLineToken::Flag(key))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::vec::Vec;

    fn test_single(input: &str, output: CmdLineToken) {
        let mut c = CmdLine::parse(input);
        assert_eq!(c.next(), Some(output));
        assert_eq!(c.next(), None);
    }

    fn test_instance(input: &str, output: Vec<CmdLineToken>)
    {
        let c = CmdLine::parse(input);
        let vec: Vec<CmdLineToken> = c.collect();
        assert_eq!(vec, output);
    }

    #[test]
    fn test_parse_flag() {
        test_single("foo", CmdLineToken::Flag("foo"));
        test_single("\"foo bar\"", CmdLineToken::Flag("foo bar"));
    }

    #[test]
    fn test_parse_kv() {
        test_single("foo=bar", CmdLineToken::KeyValuePair("foo", "bar"));
        test_single("\"foo bar\"=baz=qux", CmdLineToken::KeyValuePair("foo bar", "baz=qux"));
        test_single("\"foo bar\"=", CmdLineToken::KeyValuePair("foo bar", ""));
        test_single("\"foo bar=fun\"=\"hello world\"", CmdLineToken::KeyValuePair("foo bar=fun", "hello world"));
    }

    #[test]
    fn test_parse_multiple() {
        test_instance("foo  bar=baz   ", vec![CmdLineToken::Flag("foo"), CmdLineToken::KeyValuePair("bar", "baz")]);
        test_instance("foo  bar=baz  \"what=a fun\" ", vec![
            CmdLineToken::Flag("foo"), CmdLineToken::KeyValuePair("bar", "baz"), CmdLineToken::Flag("what=a fun")
        ]);
    }
}