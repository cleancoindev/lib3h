use errors::*;

use std::hash::Hash;

use std;

#[derive(Debug)]
pub struct PollEventState
{
    pub did_something: bool,
}

impl PollEventState
{
    pub fn new () -> Self {
        Self {
            did_something: false,
        }
    }
}

pub trait PollEvent {
    type PollEventType: Sized + std::fmt::Debug;

    fn process_once (&mut self, event_state: &mut PollEventState) -> Vec<Self::PollEventType>;
}

pub trait Endpoint: Sized + std::fmt::Display + std::fmt::Debug + Eq + Hash + Clone {
    fn new (raw: &[u8]) -> Result<Self>;
    fn get (&self) -> Result<Vec<u8>>;
}

#[derive(Debug)]
pub enum TransportEvent {
    OnError(Error),
}

pub trait Transport: Sized + PollEvent<PollEventType = TransportEvent> {
    type EndpointImpl: Endpoint;

    fn new (config: &str) -> Result<Self>;
    fn assert_connection (&mut self, endpoint: &Self::EndpointImpl);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    struct TestEndpoint {
        addr: String,
        port: u16,
    }

    impl Endpoint for TestEndpoint {
        fn new (raw: &[u8]) -> Result<Self> {
            let raw = String::from_utf8_lossy(raw);
            let parts: Vec<&str> = raw.split(':').collect();
            if parts.len() != 2 {
                return Err("bad endpoint deserialize".into());
            }
            Ok(Self {
                addr: parts[0].to_string(),
                port: parts[1].parse::<u16>()?,
            })
        }

        fn get (&self) -> Result<Vec<u8>> {
            Ok(format!("{}", self).as_bytes().to_vec())
        }
    }

    impl std::fmt::Display for TestEndpoint {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}:{}", self.addr, self.port)
        }
    }

    struct TestTransport {
        endpoints: HashMap<TestEndpoint, usize>,
    }

    impl Transport for TestTransport {
        type EndpointImpl = TestEndpoint;

        fn new (_config: &str) -> Result<Self> {
            Ok(Self {
                endpoints: HashMap::new(),
            })
        }

        fn assert_connection (&mut self, endpoint: &Self::EndpointImpl) {
            self.endpoints.insert(endpoint.clone(), 0);
        }
    }

    impl PollEvent for TestTransport {
        type PollEventType = TransportEvent;

        fn process_once (&mut self, event_state: &mut PollEventState) -> Vec<Self::PollEventType> {
            event_state.did_something = true;

            let mut events: Vec<Self::PollEventType> = Vec::new();
            events.push(TransportEvent::OnError("hello".into()));
            events
        }
    }

    #[test]
    fn it_runs () {
        let e = TestEndpoint {
            addr: "test".to_string(),
            port: 12,
        };
        println!("endpoint: {}", e);

        let mut t = TestTransport::new("").unwrap();
        t.assert_connection(&e);

        let mut state: PollEventState = PollEventState::new();
        let events = t.process_once(&mut state);

        println!("state: {:?}, events: {:?}", state, events);
    }
}
