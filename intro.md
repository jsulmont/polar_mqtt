# Flattening a Legacy C++ API for Rust Integration

> ⚠️ **Important**: This document uses **PolarMqtt**, a hypothetical library, to demonstrate API flattening concepts. It is based on [Eclipse paho.mqtt.c](https://github.com/eclipse-paho/paho.mqtt.c) but should not be used in production. For production applications, please use the official [paho-mqtt](https://crates.io/crates/paho-mqtt) crate.

## Architecture Overview

Legacy C++ APIs often present integration challenges for Rust, especially when they involve inheritance, virtual methods, and complex memory management. These challenges are addressed by introducing a C-compatible intermediate layer—a process called “flattening”—that transforms the complex C++ interface into a simpler C API that Rust can easily consume.

### Language Boundaries

Each arrow in our architecture represents a boundary where code transitions between languages and calling conventions:

```ascii
┌────────────┐     ┌────────────┐     ┌────────────┐
│   C++ API  │ --> │    C API   │ --> │  Rust API  │
│ (Original) │     │(Flattened) │     │ (Binding)  │
└────────────┘     └────────────┘     └────────────┘
     ^                  ^                   ^
     │                  │                   │
   Objects          FFI Layer           Safe Rust
   Classes          C Types             Ownership
   Inheritance      Callbacks           Lifetimes
   RAII             Pointers            Results
```

### Key Challenges Addressed

- **ABI Compatibility**: Bridging differences in vtable implementation between C++ and Rust.
- **Resource Management**: Integrating C++ RAII with Rust’s ownership model.
- **Error Handling**:  Mapping C++ exceptions to Rust’s Result type.
- **Threading Model**: Ensuring thread-safe operations across language boundaries.



## Original PolarMqtt C++ API

**PolarMqtt** is a classic C++ API providing an object-oriented interface for MQTT communications. Key components include:

### Key Components

1. **Message Class**
   - Represents MQTT messages with topic, payload, QoS, and retention settings.
   - Uses RAII for automatic resource management.
   - Provides accessor methods for message properties.

2. **Session Class**
   - Manages the MQTT connection lifecycle.
   - Handles subscriptions and message publishing.
   - Supports asynchronous callbacks for message reception, state changes and errors.

3. **Connection Configuration**
   - Fluent interface for connection settings.
   - Supports TLS/SSL configuration.
   - Handles broker URLs, credentials, and protocol settings.

### Example Usage

```cpp
// Initialize API
mqtt::APIFactory::getInstance()->initialize("MyApp", "1.0");

// Create session handler
class MySessionHandler : public mqtt::SessionHandler {
    void onStateChange(mqtt::SessionState newState) override {
        std::cout << "Connection state: " << static_cast<int>(newState) << std::endl;
    }
    void onError(int errorCode, const char* message) override {
        std::cout << "Error: " << message << std::endl;
    }
};

// Create and configure session
MySessionHandler handler;
mqtt::Session& session = mqtt::APIFactory::getInstance()->createSession("ClientID", handler);

// Configure connection
session.getConfig()
    .setBroker("broker.example.com", 1883)
    .setCredentials("username", "password")
    .set(mqtt::ConnectionConfig::Parameter::KEEP_ALIVE_INTERVAL, 60);

// Start session and publish message
session.start();
session.publish("topic/test", 
    reinterpret_cast<const uint8_t*>("Hello"), 
    5, 
    mqtt::Message::QoS::AT_LEAST_ONCE);
```

#### Design Patterns

The API employs several C++ design patterns:

- **Factory Pattern**: APIFactory for session creation.
- **Observer Pattern**: Callbacks for message and state notifications.
- **PIMPL Idiom**: Implementation hiding via opaque pointers.
- **RAII**: Automatic resource management.
- **Builder Pattern**: Fluent interface for configuration.



## Flattening: Implementation

### 1. C Interface Layer

```c
// Opaque handle representing C++ objects
typedef struct mqtt_session_t* mqtt_session_handle_t;

// Core operations
int mqtt_session_start(mqtt_session_handle_t session);
int mqtt_session_stop(mqtt_session_handle_t session);
void mqtt_destroy_session(mqtt_session_handle_t session);

// Message operations
int mqtt_publish(mqtt_session_handle_t session, 
                const char* topic,
                const uint8_t* payload, 
                size_t length,
                mqtt_qos_t qos,
                int retain);

// Callback type definitions
typedef void (*mqtt_message_callback_t)(
    const mqtt_message_data_t* message,
    void* user_context
);
```

### 2. Rust Types and Ownership

```rust
// Publishing: Rust owns the data
#[derive(Debug, Clone)]
pub struct Message {
    topic: String,
    payload: Vec<u8>,
    qos: QoS,
    retained: bool,
}

// Callbacks: Borrowing C++-owned data
#[derive(Debug)]
pub struct MessageView<'a> {
    topic: &'a str,      // References data owned by C++
    payload: &'a [u8],   // References data owned by C++
    qos: QoS,
    retained: bool,
}

// Main client interface
pub struct Client {
    session: mqtt_session_handle_t,
    callbacks: Arc<Mutex<CallbackContext>>,
}

// RAII cleanup
impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            mqtt_session_stop(self.session);
            mqtt_destroy_session(self.session);
        }
    }
}
```

#### Lifetime Parameter Explanation
The `'a` lifetime in MessageView is crucial for safety:

- During callbacks, the C++ layer owns the message data (topic and payload).
- `MessageView` borrows this data only for the duration of the callback.
- The `'a` lifetime ensures these references cannot outlive the callback.
- After the callback returns, the C++ layer may free or modify the data.

### 3. Thread-Safe Callback System

```rust
struct CallbackContext {
    // Callbacks must be Send + Sync for thread-safe invocation
    message_callback: Box<dyn Fn(&MessageView) + Send + Sync>,
    state_callback: Box<dyn Fn(ConnectionState) + Send + Sync>,
}

impl Client {
    pub fn new(
        broker_url: &str,
        on_message: impl Fn(&MessageView) + Send + Sync + 'static,
        on_state_change: impl Fn(ConnectionState) + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        let callbacks = Arc::new(Mutex::new(CallbackContext {
            message_callback: Box::new(on_message),
            state_callback: Box::new(on_state_change),
        }));
        
        // Implementation details...
        Ok(Client { session, callbacks })
    }
}

// C callback bridge
unsafe extern "C" fn message_callback(
    message: *const mqtt_message_data_t,
    context: *mut std::ffi::c_void,
) {
    let context = ManuallyDrop::new(
        Arc::from_raw(context as *const Mutex<CallbackContext>)
    );
    
    if let Ok(guard) = context.lock() {
        let msg = MessageView::from_raw(message);
        (guard.message_callback)(&msg);
    }
}
```

### 4.Error Handling Strategy

####  Core Error Types
The API implements a layered error handling strategy that bridges C++ errors into idiomatic Rust:


```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("MQTT initialization failed")]
    InitializationError,
    #[error("Invalid broker URL")]
    InvalidBrokerUrl,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Connection failed")]
    ConnectionError,
    #[error("Subscription failed")]
    SubscriptionError,
    #[error("Publication failed")]
    PublicationError,
    #[error("Invalid topic")]
    InvalidTopic,
    #[error("String contains null byte: {0}")]
    NulError(#[from] NulError),
}

pub type Result<T> = std::result::Result<T, Error>;
```

#### Error Propagation Chain

The error handling flows through three distinct layers:

- **C++ Layer**: 
   - Uses return codes for synchronous operations.
   - Provides error callbacks for asynchronous notifications.
   - Reports both error codes and descriptive messages.
   
```cpp
virtual void onError(int errorCode, const char *message) = 0;
```
- **FFI Bridge**:
   - Converts C++ errors to Rust types.
   - Manages safe string conversion from C to Rust.
   - Routes errors through callback context.

```rust
unsafe extern "C" fn error_callback(
    error_code: std::os::raw::c_int,
    message: *const std::os::raw::c_char,
    context: *mut std::ffi::c_void,
) {
    if !message.is_null() && !context.is_null() {
        let context = ManuallyDrop::new(Arc::from_raw(
            context as *const Mutex<CallbackContext>
        ));
        
        let error_msg = CStr::from_ptr(message)
            .to_str()
            .unwrap_or("Invalid error message");

        if let Ok(guard) = context.lock() {
            (guard.error_callback)(error_code, error_msg);
        }
    }
}
```
- **Rust API**:
  - Exposes errors through the Result type.
  - Maps FFI error codes to strongly-typed error enums.
  - Implements explicit error checks for FFI calls.

```rust
pub fn connect(&mut self) -> Result<()> {
    let result = unsafe { 
        bindings::mqtt_session_start(self.session) 
    };
    
    if result != 0 {
        return Err(Error::ConnectionError);
    }
    Ok(())
}
```


### 5. Thread Safety Guarantees

The Client type implements Send and Sync because:
- The underlying C++ library guarantees thread-safe operations.
- All mutable state is protected by Mutex.
- Callbacks are explicitly required to be Send + Sync.

```rust
// Safety: All mutable state is protected by Mutex,
// and the C++ library guarantees thread-safe operations
unsafe impl Send for Client {}
unsafe impl Sync for Client {}
```

## Testing Strategy

1. **Unit Tests**: Test Rust wrapper functionality

	For example: 
	
   ```rust
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
   ```

2. **Integration Tests**: Verify C++ interop
	
For a complete integration test, refer to the end of [client.rs](src/client.rs). 

_Note_: This test requires an internet connection and assumes that `broker.emqx.io` is accessible on port `1883`.

## Best Practices

1. **Memory Management**
   - Use RAII patterns in both C++ and Rust.
   - Explicitly document ownership transfer points.
   - Avoid raw pointer manipulation outside of FFI boundaries.

2. **Error Handling**
   - Convert all C++ exceptions to error codes at the C boundary.
   - Map error codes to meaningful Rust errors.
   - Provide detailed error context.

3. **Thread Safety**
   - Document thread safety guarantees.
   - Use appropriate synchronization primitives.
   - Ensure callbacks are thread-safe.

4. **Documentation**
   - Document safety requirements for unsafe functions.
   - Explain lifetime guarantees for borrowed data.
   - Provide usage examples for common scenarios.