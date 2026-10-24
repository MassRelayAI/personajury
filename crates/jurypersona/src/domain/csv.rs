//! A small RFC 4180 reader for cell locators (D21): quoted fields, doubled quotes, CRLF, line breaks inside quotes. It
//! returns the shape only, the number of fields in each record, because a citation names a cell, it does not read it.
//! Rows count records, not lines: a quoted field that holds a line break is still one row. A quote opens a quoted field
//! only at the start of a field; inside an unquoted field (`27" screen`) it is a literal character.

/// Fields per record, in order. A final line break does not start a record; an empty line is a record of one empty field.
pub fn shape(text: &str) -> Vec<usize> {
    let mut rows = Vec::new();
    let mut fields = 0usize;
    let mut in_quotes = false;
    let mut pending = false; // something of the current record has been read
    let mut at_start = true; // at the start of a field: only there does a quote open a quoted field (RFC 4180)
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        if in_quotes {
            if c == '"' {
                if it.peek() == Some(&'"') {
                    it.next();
                } else {
                    in_quotes = false;
                }
            }
            continue;
        }
        pending = true;
        let start = std::mem::replace(&mut at_start, false);
        match c {
            // a quote inside an unquoted field (`27" screen`) is a literal character
            '"' if start => in_quotes = true,
            ',' => {
                fields += 1;
                at_start = true;
            }
            '\r' | '\n' => {
                if c == '\r' && it.peek() == Some(&'\n') {
                    it.next();
                }
                rows.push(fields + 1);
                fields = 0;
                pending = false;
                at_start = true;
            }
            _ => {}
        }
    }
    if pending {
        rows.push(fields + 1);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_records() {
        assert_eq!(shape("a,b,c\n1,2,3\n"), [3, 3]);
        assert_eq!(shape("a,b,c\n1,2,3"), [3, 3]);
    }
    #[test]
    fn crlf_is_one_break() {
        assert_eq!(shape("a,b\r\n1,2\r\n"), [2, 2]);
    }
    #[test]
    fn a_quoted_comma_is_not_a_separator() {
        assert_eq!(shape("name,price\n\"Team, annual\",12\n"), [2, 2]);
    }
    #[test]
    fn doubled_quotes_stay_inside_the_field() {
        assert_eq!(shape("a,b\n\"say \"\"hi\"\", ok\",2\n"), [2, 2]);
    }
    #[test]
    fn a_line_break_inside_quotes_is_one_row() {
        assert_eq!(shape("a,b\n\"line one\nline two\",2\n3,4\n"), [2, 2, 2]);
        assert_eq!(shape("a,b\r\n\"x\r\ny\",2\r\n"), [2, 2]);
    }
    #[test]
    fn a_quote_inside_an_unquoted_field_is_literal() {
        assert_eq!(shape("name,size\n27\" screen,12\nb,7\nc,8\n"), [2, 2, 2, 2]);
        assert_eq!(shape("a,b\nx\"y\"z,1\n\"q\",2\n"), [2, 2, 2]);
    }
    #[test]
    fn ragged_rows_and_empty_fields() {
        assert_eq!(shape("a,b,c\n1,2\n,,\n"), [3, 2, 3]);
        assert_eq!(shape(""), Vec::<usize>::new());
        assert_eq!(shape("a\n\nb\n"), [1, 1, 1]);
    }
}
