use askama::Result;
use regex::Regex;
use askama::Values;



#[askama::filter_fn]
pub fn strip_html_tags(s: &str, _: &dyn Values) -> Result<String> {
    let re = Regex::new(r"<[^>]*>").unwrap();
    let without_tags = re.replace_all(s, "").to_string();

    let replacements = [
        ("&nbsp;", " "),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&amp;", "&"),
        ("&quot;", "\""),
        ("&apos;", "'"),
    ];

    let mut result = without_tags;
    for &(entity, replacement) in &replacements {
        let re_entity = Regex::new(entity).unwrap();
        result = re_entity.replace_all(&result, replacement).to_string();
    }

    Ok(result)
}