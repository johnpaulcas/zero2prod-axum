use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if s.validate_email() {
            Ok(Self(s))
        } else {
            Err(format!("invalid email {}", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use claims::{assert_err, assert_ok};
    use fake::{Fake, faker::internet::ar_sa::FreeEmail};

    #[test]
    fn valid_email_passed_successfully() {
        let email = FreeEmail().fake();
        assert_ok!(SubscriberEmail::parse(email));
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_no_symbol_rejected() {
        let email = "paul.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_no_subject_rejected() {
        let email = "@gmail.com";
        assert_err!(SubscriberEmail::parse(email.to_string()));
    }
}
