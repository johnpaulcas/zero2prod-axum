use crate::domain::{SubsciberName, SubscriberEmail};

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubsciberName,
}
