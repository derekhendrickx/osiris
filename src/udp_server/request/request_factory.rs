use crate::udp_server::{action::Action, payload::Payload};

struct RequestFactory;

impl RequestFactory {
    fn new_payload(action: &Action) -> Option<Box<dyn Payload>> {
        match action {
            Action::Connect => None,
            Action::Announce => todo!(),
            Action::Scrape => todo!(),
            Action::Error => todo!(),
        }
    }
}
