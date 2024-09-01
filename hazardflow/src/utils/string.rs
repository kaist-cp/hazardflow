//! String utilities.

/// Combines all elements into one String, separated by `sep`. Returns `None` if all elements are `None`.
// TODO: Make this function macro
pub fn join_options<I>(sep: &str, iterable: I) -> Option<String>
where I: IntoIterator<Item = Option<String>> {
    let iterable = iterable.into_iter().flatten().collect::<Vec<_>>();
    if iterable.is_empty() {
        None
    } else {
        Some(iterable.join(sep))
    }
}

/// Indents every line in the string.
pub fn indent(str: String, indent: usize) -> String {
    str.lines().map(|l| format!("{}{}", " ".repeat(indent), l)).collect::<Vec<_>>().join("\n")
}
