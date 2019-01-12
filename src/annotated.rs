pub fn annotate<T, U>(items: Vec<T>, make_annotation: fn(&T) -> U) -> Vec<(T, U)> {
    items.into_iter()
        .map(|item| {
            let annotation = make_annotation(&item);
            (item, annotation)
        })
        .collect()
}


pub fn to_parts(s: &str) -> Vec<Part> {
    // While it's unlikely the empty input fast path will be triggered here,
    // knowing the input is non-empty simplifies some code below.
    if s.len() < 1 {
        return Vec::with_capacity(0); // does not allocate
    }

    // Both these passes should probably use str::find because it may
    // be implemented in platform specific assembly or other cleverness,
    // but the implementation is more complex, so maybe later. --SK

    // Preallocate. Even when a prepass is required, it's usually worth,
    // although I haven't tested in this case. However, I suspect the
    // best way here is some data analysis: we could write a script
    // to find maybe the 95th percentile count for tweets we care about
    // and hard code that. --SK

    let mut n = 0;
    // prev is initialized so the first character will always trigger
    // an increment because this doesn't process end of string.
    // Seems like there ought to be a way to get the first UTF-8 character
    // from a string without so many extraneous concepts. --SK
    let mut prev = !s.chars().next().unwrap().is_whitespace();
    for c in s.chars() {
        if prev != c.is_whitespace() {
            n += 1;
            prev = !prev;
        }
    }
    // println!("{} {}", n, s);

    let mut parts: Vec<Part> = Vec::with_capacity(n);
    let mut prev_white = s.chars().next().unwrap().is_whitespace();
    let mut beg = 0;
    let mut iter = s.char_indices();
    loop {
        // Throwing the push in a function and calling it from both inside
        // and after the loop would probably be better than ripping a simple
        // loop up like this. --SK
        let (i, cur_white) = match iter.next() {
            Some((i, c)) => (i, c.is_whitespace()),
            None => (s.len(), !prev_white),
        };
        if prev_white != cur_white {
            let end = i;
            let sub = &s[beg..end];
            parts.push(if prev_white {
                Part::Whitespace(sub)
            } else {
                Part::Word(sub)
            });
            beg = end;
            prev_white = !prev_white;
        }
        if i >= s.len() {
            break
        }
    }

    parts
}

#[cfg(test)]
mod test {
    use super::{Part::*, *};

    #[test]
    fn to_parts_ascii() {
        assert_eq!(to_parts(""), []);
        assert_eq!(to_parts("  "), [Whitespace("  ")]);
        assert_eq!(to_parts("one-word"), [Word("one-word")]);
        assert_eq!(to_parts("alpha  bet ic"), [Word("alpha"), Whitespace("  "), Word("bet"), Whitespace(" "), Word("ic")]);
        assert_eq!(to_parts("alpha  bet ic   "), [Word("alpha"), Whitespace("  "), Word("bet"), Whitespace(" "), Word("ic"), Whitespace("   ")]);
        assert_eq!(to_parts(" alpha  bet ic   "), [Whitespace(" "), Word("alpha"), Whitespace("  "), Word("bet"), Whitespace(" "), Word("ic"), Whitespace("   ")]);
    }

    #[test]
    fn annotate_length() {
        assert_eq!(
            annotate(
                to_parts("alpha  bet ic"),
                |part| part.get_string().len()
            ),
            [
                (Word("alpha"), 5),
                (Whitespace("  "), 2),
                (Word("bet"), 3),
                (Whitespace(" "), 1),
                (Word("ic"), 2),
            ]
        );
    }
}

#[derive(Debug, PartialEq)]
pub enum Part<'a> {
    Word(&'a str),
    Whitespace(&'a str),
}

impl<'a> Part<'a> {
    fn get_string(&self) -> &'a str {
        match self {
            Part::Word(s) | Part::Whitespace(s) => s
        }
    }
}
