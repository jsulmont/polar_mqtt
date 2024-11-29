#ifndef MQTT_C_BINDINGS_H
#define MQTT_C_BINDINGS_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C"
{
#endif

    // Opaque handle types - only for sessions which we do own
    typedef struct mqtt_session_t *mqtt_session_handle_t;

    // Message data struct - used to pass message data during callbacks
    typedef struct mqtt_message_data_t
    {
        const char *topic;
        const uint8_t *payload;
        size_t payload_length;
        int32_t qos;
        int32_t retained;
        int64_t message_id;
    } mqtt_message_data_t;

    typedef enum mqtt_qos_t
    {
        MQTT_QOS_AT_MOST_ONCE = 0,
        MQTT_QOS_AT_LEAST_ONCE = 1,
        MQTT_QOS_EXACTLY_ONCE = 2
    } mqtt_qos_t;

    typedef enum mqtt_session_state_t
    {
        MQTT_STATE_DISCONNECTED = 0,
        MQTT_STATE_CONNECTING = 1,
        MQTT_STATE_CONNECTED = 2,
        MQTT_STATE_RECONNECTING = 3
    } mqtt_session_state_t;

    typedef enum mqtt_parameter_t
    {
        MQTT_PARAM_KEEP_ALIVE_INTERVAL = 0,
        MQTT_PARAM_CLEAN_SESSION = 1,
        MQTT_PARAM_CONNECTION_TIMEOUT = 2,
        MQTT_PARAM_MAX_INFLIGHT = 3,
        MQTT_PARAM_MAX_QUEUED_MESSAGES = 4,
        MQTT_PARAM_RECONNECT_DELAY = 5,
        MQTT_PARAM_TLS_ENABLED = 6
    } mqtt_parameter_t;

    // Callback function types
    // Note: Message data is only valid during callback execution
    typedef void (*mqtt_message_callback_t)(const mqtt_message_data_t *message, void *user_context);
    typedef void (*mqtt_state_callback_t)(mqtt_session_state_t new_state, void *user_context);
    typedef void (*mqtt_error_callback_t)(int error_code, const char *message, void *user_context);

    // Session configuration functions
    int mqtt_set_int_parameter(mqtt_session_handle_t session, mqtt_parameter_t param, int32_t value);
    int mqtt_set_bool_parameter(mqtt_session_handle_t session, mqtt_parameter_t param, int value);
    int mqtt_set_broker(mqtt_session_handle_t session, const char *url, uint16_t port);
    int mqtt_set_credentials(mqtt_session_handle_t session, const char *username, const char *password);
    int mqtt_set_tls_certificates(mqtt_session_handle_t session, const char *ca_file,
                                  const char *cert_file, const char *key_file);

    // Session lifecycle functions
    int mqtt_initialize(const char *app_name, const char *app_version, int debug, const char *log_file);
    int mqtt_uninitialize(void);
    mqtt_session_handle_t mqtt_create_session(const char *client_id,
                                              mqtt_message_callback_t message_cb,
                                              mqtt_state_callback_t state_cb,
                                              mqtt_error_callback_t error_cb,
                                              void *user_context);
    void mqtt_destroy_session(mqtt_session_handle_t session);

    // Session control functions
    mqtt_session_state_t mqtt_session_get_state(mqtt_session_handle_t session);
    int mqtt_session_start(mqtt_session_handle_t session);
    int mqtt_session_stop(mqtt_session_handle_t session);

    // Subscription functions
    int64_t mqtt_subscribe(mqtt_session_handle_t session, const char *topic, mqtt_qos_t qos);
    int mqtt_unsubscribe(mqtt_session_handle_t session, int64_t handle);

    // Publishing functions
    int64_t mqtt_publish(mqtt_session_handle_t session, const char *topic,
                         const uint8_t *payload, size_t length,
                         mqtt_qos_t qos, int retain);

#ifdef __cplusplus
}
#endif

#endif // MQTT_C_BINDINGS_H