use crate::bindings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl From<QoS> for bindings::mqtt_qos_t {
    fn from(qos: QoS) -> Self {
        match qos {
            QoS::AtMostOnce => bindings::mqtt_qos_t_MQTT_QOS_AT_MOST_ONCE,
            QoS::AtLeastOnce => bindings::mqtt_qos_t_MQTT_QOS_AT_LEAST_ONCE,
            QoS::ExactlyOnce => bindings::mqtt_qos_t_MQTT_QOS_EXACTLY_ONCE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

impl From<bindings::mqtt_session_state_t> for ConnectionState {
    fn from(state: bindings::mqtt_session_state_t) -> Self {
        match state {
            bindings::mqtt_session_state_t_MQTT_STATE_DISCONNECTED => ConnectionState::Disconnected,
            bindings::mqtt_session_state_t_MQTT_STATE_CONNECTING => ConnectionState::Connecting,
            bindings::mqtt_session_state_t_MQTT_STATE_CONNECTED => ConnectionState::Connected,
            bindings::mqtt_session_state_t_MQTT_STATE_RECONNECTING => ConnectionState::Reconnecting,
            _ => ConnectionState::Disconnected,
        }
    }
}
