use tui::text::Text;
use unicode_segmentation::UnicodeSegmentation;

pub fn create_text(content: &str, wrap_width: usize) -> Text {
    if content.len() > wrap_width {
        let indices = wrap_indices(content, wrap_width);
        assert!(!indices.is_empty());

        let lines = split_string_at_indices(content, &indices);
        let mut line_iter = lines.iter();

        let mut text = Text::from(*line_iter.next().unwrap());
        for line in line_iter {
            text.extend(Text::from(*line));
        }
        text
    } else {
        Text::from(content)
    }
}

fn wrap_indices(text: &str, max_width: usize) -> Vec<usize> {
    let word_indices = text.split_word_bound_indices().map(|(pos, _)| pos);

    let mut lines = vec![];
    let mut prev = None;
    let mut len = max_width;

    for pos in word_indices.chain(std::iter::once(text.len())) {
        if pos > len {
            if let Some(prev) = prev {
                lines.push(prev)
            }
            prev = Some(pos);
            len += max_width;
        } else {
            prev = Some(pos)
        }
    }

    lines
}

fn split_string_at_indices<'a>(s: &'a str, indices: &[usize]) -> Vec<&'a str> {
    assert!(*indices.iter().max().unwrap_or(&0) < s.len());

    let mut off = 0_usize;
    let mut ms = s;
    let mut parts: Vec<&str> = indices
        .iter()
        .map(|&index| {
            let (head, tail) = ms.split_at((index - off) as usize);
            off = index;
            ms = tail;
            head
        })
        .collect();
    parts.push(ms);
    parts
}

#[test]
fn wrap_long_text() {
    let text = concat!(
        "Explicit concurrent copying GC freed 47311(2322KB) AllocSpace objects, ",
        "17(724KB) LOS objects, 49% free, 12MB/25MB, paused 339us total 141.468ms"
    );

    assert_eq!(wrap_indices(text, 20), vec![20, 37, 51, 80, 98, 115, 134]);
}

#[test]
fn wrap_short_text() {
    let text = "Lorem ipsum";

    assert_eq!(wrap_indices(text, 20), vec![]);
}

#[test]
fn wrap_empty_text() {
    let text = "";

    assert_eq!(wrap_indices(text, 20), vec![]);
}

#[test]
fn test_split_string_at_indices() {
    let s = concat!(
        "Explicit concurrent copying GC freed 47311(2322KB) AllocSpace objects, ",
        "17(724KB) LOS objects, 49% free, 12MB/25MB, paused 339us total 141.468ms"
    );
    let indices = wrap_indices(s, 20);

    let splits = split_string_at_indices(s, &indices);

    assert_eq!(
        splits,
        vec![
            "Explicit concurrent ",
            "copying GC freed ",
            "47311(2322KB) ",
            "AllocSpace objects, 17(724KB)",
            " LOS objects, 49% ",
            "free, 12MB/25MB, ",
            "paused 339us total ",
            "141.468ms"
        ]
    );
}

#[test]
fn test_split_string_at_no_indices() {
    let s = "Explicit concurrent copying";
    let splits = split_string_at_indices(s, &[]);

    assert_eq!(splits, vec!["Explicit concurrent copying"]);
}

#[test]
fn test_split_suspicious() {
    let s = concat!(
        "Invalidating LocalCallingIdentity cache for package ",
        "com.tomtom.ivi.functionaltest.frontend.alexa.test. ",
        "Reason: package android.intent.action.PACKAGE_REMOVED"
    );
    let indices = wrap_indices(s, 50);

    assert_eq!(indices, vec![44, 52, 119]);
}

#[test]
fn multiline_text() {
    let s = concat!(
        "Invalidating LocalCallingIdentity cache for package ",
        "com.tomtom.ivi.functionaltest.frontend.alexa.test. ",
        "Reason: package android.intent.action.PACKAGE_REMOVED"
    )
    .to_string();

    let text = create_text(s.as_str(), 50);

    assert_eq!(4, text.height());
    assert_eq!(
        text.lines
            .iter()
            .flat_map(|spans| { spans.0.iter().map(|span| span.content.to_string()) })
            .collect::<Vec<_>>()
            .join("$"),
        concat!(
            "Invalidating LocalCallingIdentity cache for $",
            "package $",
            "com.tomtom.ivi.functionaltest.frontend.alexa.test. Reason: package $",
            "android.intent.action.PACKAGE_REMOVED"
        )
    )
}
