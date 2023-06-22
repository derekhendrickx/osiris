use super::{payload::Payload, request::headers::Headers};

struct TrackerRequest {
    headers: Headers,
    payload: Option<Box<dyn Payload>>,
}

#[cfg(test)]
mod tests {
    use fake::{Faker, Fake};

    use crate::udp_server::{tracker_request::TrackerRequest, request::headers::{PROTOCOL_ID, Headers}, action::Action};

    #[test]
    fn test_it_creates_a_tracker_request() {
        let connection_id = PROTOCOL_ID;
        let action = Action::Connect;
        let transaction_id = Faker.fake::<u32>();
        let headers = Headers::new(
            connection_id,
            action,
            transaction_id
        );

        let tracker_request = TrackerRequest {
            headers: headers,
            payload: None,
        };

        assert_eq!(tracker_request.headers.connection_id(), connection_id);
        assert_eq!(tracker_request.headers.action(), &Action::Connect);
        assert_eq!(tracker_request.headers.transaction_id(), transaction_id);
        assert!(tracker_request.payload.is_none());
    }
}