use crate::state::LineEnding;

pub struct LineParser {
    buffer: String,
    mode: LineEnding,
}

impl LineParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            mode: LineEnding::NL,
        }
    }

    pub fn set_mode(&mut self, mode: LineEnding) {
        self.mode = mode;
    }

    /// Appends new data and returns any complete lines parsed according to the current mode.
    pub fn push(&mut self, data: &str) -> Vec<String> {
        let mut lines = Vec::new();
        self.buffer.push_str(data);

        match self.mode {
            LineEnding::None => {
                // Raw mode: Flush everything immediately
                if !self.buffer.is_empty() {
                    lines.push(self.buffer.clone());
                    self.buffer.clear();
                }
            }
            LineEnding::NL => {
                // Split by '\n'
                while let Some(pos) = self.buffer.find('\n') {
                    let mut line: String = self.buffer.drain(..=pos).collect();
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    if line.ends_with('\r') {
                        line.pop();
                    } // Handle potential \r before \n
                    lines.push(line);
                }
            }
            LineEnding::CR => {
                // Split by '\r'
                while let Some(pos) = self.buffer.find('\r') {
                    let mut line: String = self.buffer.drain(..=pos).collect();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                    lines.push(line);
                }
            }
            LineEnding::NLCR => {
                // Split by "\r\n"
                while let Some(pos) = self.buffer.find("\r\n") {
                    // pos is start of \r\n. Length is 2.
                    // drain range ..=(pos+1) covers \r(pos) and \n(pos+1)
                    let mut line: String = self.buffer.drain(..=pos + 1).collect();
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    if line.ends_with('\r') {
                        line.pop();
                    }
                    lines.push(line);
                }
            }
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_mode() {
        let mut parser = LineParser::new();
        parser.set_mode(LineEnding::None);
        assert_eq!(parser.push("hello"), vec!["hello"]);
        assert_eq!(parser.push("world"), vec!["world"]);
    }

    #[test]
    fn test_nl_mode() {
        let mut parser = LineParser::new();
        parser.set_mode(LineEnding::NL);

        let lines = parser.push("hello\nworld");
        assert_eq!(lines, vec!["hello"]);

        let lines = parser.push("\n");
        assert_eq!(lines, vec!["world"]);

        // \r before \n is stripped
        assert_eq!(parser.push("windows\r\n"), vec!["windows"]);
    }

    #[test]
    fn test_cr_mode() {
        let mut parser = LineParser::new();
        parser.set_mode(LineEnding::CR);

        assert_eq!(parser.push("one\rtwo\r"), vec!["one", "two"]);
        assert_eq!(parser.push("thr"), Vec::<String>::new());
        assert_eq!(parser.push("ee\r"), vec!["three"]);
    }

    #[test]
    fn test_nlcr_mode() {
        let mut parser = LineParser::new();
        parser.set_mode(LineEnding::NLCR);

        assert_eq!(parser.push("one\r\ntwo"), vec!["one"]);
        assert_eq!(parser.push("\r\n"), vec!["two"]);

        assert_eq!(parser.push("half\r"), Vec::<String>::new());
        assert_eq!(parser.push("\n"), vec!["half"]);
    }
}
