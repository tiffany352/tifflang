use std::rc::Rc;
use std::fmt;

#[derive(Clone)]
pub struct Span<T> {
    pub value: T,
    first: usize,
    last: usize,
    buffer: Rc<String>,
}

impl<T> Span<T> {
    pub fn new(value: T, first: usize, last: usize, buffer: Rc<String>) -> Span<T> {
        Span {
            value: value,
            first: first,
            last: last,
            buffer: buffer,
        }
    }

    pub fn map<U>(&self, func: fn(value: &T) -> U) -> Span<U> {
        Span {
            value: func(&self.value),
            first: self.first,
            last: self.last,
            buffer: self.buffer.clone(),
        }
    }

    pub fn replace<U>(&self, value: U) -> Span<U> {
        Span {
            value: value,
            first: self.first,
            last: self.last,
            buffer: self.buffer.clone(),
        }
    }

    pub fn split(self) -> (Span<()>, T) {
        let span = self.replace(());
        (span, self.value)
    }

    pub fn bridge<U, V>(first: Span<U>, last: Span<V>, value: T) -> Span<T> {
        Span {
            value: value,
            first: first.first,
            last: last.last,
            buffer: first.buffer.clone(),
        }
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }
}

impl<T: fmt::Display> fmt::Display for Span<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "{}", self.value)?;
        let first_line_start = self.buffer[..self.first].rfind('\n').map(|index| index + 1).unwrap_or(0);
        let last_line_end = self.buffer[self.last..].find('\n').map(|index| index + self.last).unwrap_or(self.buffer.len());
        let mut line_start = first_line_start;
        for line in self.buffer[first_line_start..last_line_end].lines() {
            writeln!(fmt, "--> {}", line)?;
            let first = if self.first > line_start { self.first - line_start } else { 0 };
            let last = (self.last - line_start).min(line.len());
            writeln!(
                fmt,
                "    {: >spaces$}{:^>carets$}",
                "", "",
                spaces = first,
                carets = if last > first { last - first } else { 0 },
            )?;
            line_start += line.len() + 1;
        }
        Ok(())
    }
}

impl<T: fmt::Debug> fmt::Debug for Span<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(fmt)?;
        let line = self.buffer[..self.first].matches('\n').count() + 1;
        let first_line_start = self.buffer[..self.first].rfind('\n').map(|index| index + 1).unwrap_or(0);
        write!(fmt, r#" [{}:{} "{}"]"#, line, (self.first - first_line_start) + 1, &self.buffer[self.first..self.last])
    }
}
