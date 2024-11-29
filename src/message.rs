use crate::QoS;

// The owned version for publishing
#[derive(Debug, Clone)]
pub struct Message {
    pub(crate) topic: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) qos: QoS,
    pub(crate) retained: bool,
}

// The borrowed version for callbacks
#[derive(Debug)]
pub struct MessageView<'a> {
    pub(crate) topic: &'a str,
    pub(crate) payload: &'a [u8],
    pub(crate) qos: QoS,
    pub(crate) retained: bool,
}

impl Message {
    pub fn new<T: Into<String>, P: Into<Vec<u8>>>(topic: T, payload: P) -> Self {
        Self {
            topic: topic.into(),
            payload: payload.into(),
            qos: QoS::AtMostOnce,
            retained: false,
        }
    }

    pub fn with_qos(mut self, qos: QoS) -> Self {
        self.qos = qos;
        self
    }

    pub fn with_retain(mut self, retained: bool) -> Self {
        self.retained = retained;
        self
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn qos(&self) -> QoS {
        self.qos
    }

    pub fn is_retained(&self) -> bool {
        self.retained
    }
}

impl MessageView<'_> {
    pub fn to_owned(&self) -> Message {
        Message {
            topic: self.topic.to_string(),
            payload: self.payload.to_vec(),
            qos: self.qos,
            retained: self.retained,
        }
    }

    pub fn topic(&self) -> &str {
        self.topic
    }

    pub fn payload(&self) -> &[u8] {
        self.payload
    }

    pub fn qos(&self) -> QoS {
        self.qos
    }

    pub fn is_retained(&self) -> bool {
        self.retained
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message {
            topic: "test/topic".into(),
            payload: vec![1, 2, 3],
            qos: QoS::AtLeastOnce,
            retained: false,
        };
        assert_eq!(msg.topic, "test/topic");
    }

    #[test]
    fn test_message_view_getters() {
        let topic = "test/topic";
        let payload = vec![1, 2, 3];
        let qos = QoS::AtLeastOnce;
        let retained = true;

        let view = MessageView {
            topic,
            payload: &payload,
            qos,
            retained,
        };

        assert_eq!(view.topic(), "test/topic");
        assert_eq!(view.payload(), &[1, 2, 3]);
        assert_eq!(view.qos(), QoS::AtLeastOnce);
        assert!(view.is_retained());
    }

    #[test]
    fn test_message_view_to_owned() {
        let topic = "test/topic";
        let payload = vec![1, 2, 3];
        let qos = QoS::AtLeastOnce;
        let retained = true;

        let view = MessageView {
            topic,
            payload: &payload,
            qos,
            retained,
        };

        let owned = view.to_owned();

        // Verify all fields are correctly converted
        assert_eq!(owned.topic(), view.topic());
        assert_eq!(owned.payload(), view.payload());
        assert_eq!(owned.qos(), view.qos());
        assert_eq!(owned.is_retained(), view.is_retained());

        // Verify we actually have owned data
        assert_eq!(owned.topic, String::from("test/topic"));
        assert_eq!(owned.payload, vec![1, 2, 3]);
    }
}
