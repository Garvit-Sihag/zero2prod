use crate::domain::{SubscriberEmail, SubscriberName};

#[derive(Debug)]
pub struct NewSubsriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}
