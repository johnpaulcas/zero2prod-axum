use unicode_segmentation::UnicodeSegmentation;

pub struct SubsciberName(String);

impl SubsciberName {
    pub fn parse(s: String) -> Result<SubsciberName, String> {
        let is_empty = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 255;

        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|c| forbidden_chars.contains(&c));

        if is_empty || is_too_long || contains_forbidden_chars {
            Err(format!("{} is not valid subscriber name", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubsciberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
