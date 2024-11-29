use crate::bindings;
use crate::error::{Error, Result};
use crate::message::{Message, MessageView};
use crate::types::{ConnectionState, QoS};
use std::ffi::{CStr, CString};
use std::mem::ManuallyDrop;
use std::sync::Once;
use std::sync::{Arc, Mutex};

static INIT: Once = Once::new();

pub type MessageCallback = dyn Fn(&MessageView) + Send + Sync;
pub type StateCallback = dyn Fn(ConnectionState) + Send + Sync;
pub type ErrorCallback = dyn Fn(i32, &str) + Send + Sync;

struct CallbackContext {
    message_callback: Box<MessageCallback>,
    state_callback: Box<StateCallback>,
    error_callback: Box<ErrorCallback>,
}

pub struct Client {
    session: *mut bindings::mqtt_session_t,
    _context: Arc<Mutex<CallbackContext>>, // Keep the context alive.
}

impl Client {
    pub fn new<F1, F2, F3>(
        client_id: &str,
        on_message: F1,
        on_state_change: F2,
        on_error: F3,
    ) -> Result<Self>
    where
        F1: Fn(&MessageView) + Send + Sync + 'static,
        F2: Fn(ConnectionState) + Send + Sync + 'static,
        F3: Fn(i32, &str) + Send + Sync + 'static,
    {
        // Initialize API once
        INIT.call_once(|| {
            let app_name = CString::new("RustMQTTClient").expect("Invalid app name");
            let app_version = CString::new("1.0").expect("Invalid version string");
            let debug = 0;
            let log_file = std::ptr::null();
            unsafe {
                bindings::mqtt_initialize(app_name.as_ptr(), app_version.as_ptr(), debug, log_file);
            }
        });

        let client_id = CString::new(client_id)?;
        let callback_context = Arc::new(Mutex::new(CallbackContext {
            message_callback: Box::new(on_message),
            state_callback: Box::new(on_state_change),
            error_callback: Box::new(on_error),
        }));

        let context_for_c = Arc::clone(&callback_context);
        let context_ptr = Arc::into_raw(context_for_c) as *mut std::ffi::c_void;

        let session = unsafe {
            bindings::mqtt_create_session(
                client_id.as_ptr(),
                Some(Self::message_callback),
                Some(Self::state_callback),
                Some(Self::error_callback),
                context_ptr,
            )
        };

        if session.is_null() {
            unsafe { Arc::from_raw(context_ptr as *const Mutex<CallbackContext>) };
            return Err(Error::InitializationError);
        }

        Ok(Self {
            session,
            _context: callback_context,
        })
    }

    pub fn connect(&mut self, host: &str, port: u16) -> Result<()> {
        let broker_host = CString::new(host)?;

        let result = unsafe { bindings::mqtt_set_broker(self.session, broker_host.as_ptr(), port) };

        if result != 0 {
            return Err(Error::InvalidBrokerUrl);
        }

        let result = unsafe { bindings::mqtt_session_start(self.session) };

        if result != 0 {
            return Err(Error::ConnectionError);
        }

        Ok(())
    }

    pub fn subscribe(&self, topic: &str, qos: QoS) -> Result<i64> {
        let topic = CString::new(topic)?;

        let handle = unsafe { bindings::mqtt_subscribe(self.session, topic.as_ptr(), qos.into()) };

        if handle < 0 {
            Err(Error::SubscriptionError)
        } else {
            Ok(handle)
        }
    }

    pub fn unsubscribe(&self, handle: i64) -> Result<()> {
        let result = unsafe { bindings::mqtt_unsubscribe(self.session, handle) };

        if result != 0 {
            Err(Error::SubscriptionError)
        } else {
            Ok(())
        }
    }

    pub fn publish(&self, message: &Message) -> Result<i64> {
        let topic = CString::new(&*message.topic)?;

        let message_id = unsafe {
            bindings::mqtt_publish(
                self.session,
                topic.as_ptr(),
                message.payload.as_ptr(),
                message.payload.len(),
                message.qos.into(),
                message.retained as i32,
            )
        };

        if message_id < 0 {
            Err(Error::PublicationError)
        } else {
            Ok(message_id)
        }
    }

    pub fn state(&self) -> ConnectionState {
        let state = unsafe { bindings::mqtt_session_get_state(self.session) };
        state.into()
    }

    unsafe extern "C" fn message_callback(
        message: *const bindings::mqtt_message_data_t,
        context: *mut std::ffi::c_void,
    ) {
        if message.is_null() || context.is_null() {
            return;
        }

        // Create a temporary reference without taking ownership
        let context = ManuallyDrop::new(Arc::from_raw(context as *const Mutex<CallbackContext>));

        let payload = if (*message).payload.is_null() || (*message).payload_length == 0 {
            &[]
        } else if (*message).payload_length > isize::MAX as usize {
            eprintln!("Payload too large");
            return;
        } else {
            std::slice::from_raw_parts((*message).payload, (*message).payload_length)
        };

        let topic = match CStr::from_ptr((*message).topic).to_str() {
            Ok(s) => s,
            Err(_) => return,
        };

        let qos = match (*message).qos {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => return,
        };

        let msg = MessageView {
            topic,
            payload,
            qos,
            retained: (*message).retained != 0,
        };

        if let Ok(guard) = context.lock() {
            (guard.message_callback)(&msg);
        };
    }

    unsafe extern "C" fn state_callback(
        state: bindings::mqtt_session_state_t,
        context: *mut std::ffi::c_void,
    ) {
        if !context.is_null() {
            let context =
                ManuallyDrop::new(Arc::from_raw(context as *const Mutex<CallbackContext>));

            if let Ok(guard) = context.lock() {
                (guard.state_callback)(state.into());
            };
        }
    }

    unsafe extern "C" fn error_callback(
        error_code: std::os::raw::c_int,
        message: *const std::os::raw::c_char,
        context: *mut std::ffi::c_void,
    ) {
        if !message.is_null() && !context.is_null() {
            let context =
                ManuallyDrop::new(Arc::from_raw(context as *const Mutex<CallbackContext>));
            let error_msg = CStr::from_ptr(message)
                .to_str()
                .unwrap_or("Invalid error message");

            if let Ok(guard) = context.lock() {
                (guard.error_callback)(error_code, error_msg);
            };
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            bindings::mqtt_session_stop(self.session);
            bindings::mqtt_destroy_session(self.session);
        }
    }
}

unsafe impl Send for Client {}
unsafe impl Sync for Client {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_integration() {
        let (tx, rx) = mpsc::channel();
        let (error_tx, error_rx) = mpsc::channel();
        let tx = Arc::new(Mutex::new(tx));
        let error_tx = Arc::new(Mutex::new(error_tx));

        let test_topic = format!("test/topic/{}", uuid::Uuid::new_v4());
        let test_topic_clone = test_topic.clone();

        let mut client = Client::new(
            &format!("TestClient_{}", uuid::Uuid::new_v4()),
            move |msg| {
                if msg.topic() == test_topic_clone {
                    let _ = tx.lock().unwrap().send(
                        Message::new(msg.topic(), msg.payload().to_vec())
                            .with_qos(msg.qos())
                            .with_retain(msg.is_retained()),
                    );
                }
            },
            |state| eprintln!("State: {:?}", state),
            move |code, err| {
                let _ = error_tx.lock().unwrap().send((code, err.to_string()));
            },
        )
        .unwrap();

        let check_errors = || {
            if let Ok((code, err)) = error_rx.try_recv() {
                panic!("MQTT error: {} - {}", code, err);
            }
        };

        client.connect("broker.emqx.io", 1883).unwrap();
        check_errors();

        client.subscribe(&test_topic, QoS::AtLeastOnce).unwrap();
        check_errors();
        thread::sleep(Duration::from_secs(1));
        check_errors();

        let message = Message::new(&test_topic, b"test").with_qos(QoS::AtLeastOnce);
        client.publish(&message).unwrap();
        check_errors();

        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(received) => {
                check_errors();
                assert_eq!(received.payload(), message.payload());
                assert_eq!(received.qos(), message.qos());
            }
            Err(e) => panic!("Receive timeout: {}", e),
        }
        check_errors();
    }
}
