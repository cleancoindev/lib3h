use errors::*;

use std::hash::Hash;

use std;

#[derive(Debug)]
pub struct PollEventState {
    pub did_something: bool,
}

impl Default for PollEventState {
    fn default() -> Self {
        Self {
            did_something: false,
        }
    }
}

impl PollEventState {
    pub fn new() -> Self {
        Default::default()
    }
}

pub trait PollEvent {
    type PollEventType: Sized + std::fmt::Debug;

    fn process_once(&mut self, event_state: &mut PollEventState) -> Vec<Self::PollEventType>;
}

pub trait Endpoint: Sized + std::fmt::Display + std::fmt::Debug + Eq + Hash + Clone {
    fn new(raw: &[u8]) -> Result<Self>;
    fn get(&self) -> Result<&[u8]>;
}

#[derive(Debug)]
pub enum TransportEvent<U> {
    OnError(Error),
    OnConnectionError(U, Error),
}

pub trait Transport<U: std::fmt::Debug>: Sized + PollEvent<PollEventType = TransportEvent<U>> {
    type EndpointImpl: Endpoint;

    fn new(config: &str) -> Result<Self>;

    fn connect(&mut self, endpoint: &Self::EndpointImpl);

    fn list_connections(&self) -> Vec<&Self::EndpointImpl>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    struct TestEndpoint {
        addr: String,
        port: u16,
        disp: String,
    }

    impl TestEndpoint {
        fn priv_new(addr: String, port: u16) -> Result<Self> {
            let disp = format!("{}:{}", addr, port);
            Ok(Self {
                addr,
                port,
                disp,
            })
        }
    }

    impl Endpoint for TestEndpoint {
        fn new(raw: &[u8]) -> Result<Self> {
            let raw = String::from_utf8_lossy(raw);
            let parts: Vec<&str> = raw.split(':').collect();
            if parts.len() != 2 {
                return Err("bad endpoint deserialize".into());
            }
            let addr = parts[0].to_string();
            let port = parts[1].parse::<u16>()?;
            TestEndpoint::priv_new(addr, port)
        }

        fn get(&self) -> Result<&[u8]> {
            Ok(self.disp.as_bytes())
        }
    }

    impl std::fmt::Display for TestEndpoint {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.disp)
        }
    }

    struct TestTransport {
        endpoints: HashMap<TestEndpoint, usize>,
    }

    impl Transport<String> for TestTransport {
        type EndpointImpl = TestEndpoint;

        fn new(_config: &str) -> Result<Self> {
            Ok(Self {
                endpoints: HashMap::new(),
            })
        }

        fn connect(&mut self, endpoint: &Self::EndpointImpl) {
            self.endpoints.insert(endpoint.clone(), 0);
        }

        fn list_connections(&self) -> Vec<&Self::EndpointImpl> {
            self.endpoints.keys().collect()
        }
    }

    impl PollEvent for TestTransport {
        type PollEventType = TransportEvent<String>;

        fn process_once(&mut self, event_state: &mut PollEventState) -> Vec<Self::PollEventType> {
            event_state.did_something = true;

            let mut events: Vec<Self::PollEventType> = Vec::new();
            events.push(TransportEvent::OnError("hello".into()));
            events
        }
    }

    #[test]
    fn it_runs() {
        let e = TestEndpoint::priv_new("test".to_string(), 12).unwrap();
        println!("endpoint: {}", e);

        let mut t = TestTransport::new("").unwrap();
        t.connect(&e);

        let mut state: PollEventState = PollEventState::new();
        let events = t.process_once(&mut state);

        println!("state: {:?}, events: {:?}", state, events);
    }
}
