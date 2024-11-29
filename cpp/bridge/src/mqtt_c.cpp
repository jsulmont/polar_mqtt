#include "mqtt_c.hpp"
#include "PolarMqtt.hpp"
#include <memory>
#include <unordered_map>
#include <mutex>

struct mqtt_session_t
{
    mqtt::Session *session;
    mqtt_message_callback_t message_cb;
    mqtt_state_callback_t state_cb;
    mqtt_error_callback_t error_cb;
    void *user_context;
};

namespace
{
    mqtt::APIFactory *g_factory = nullptr;
    std::mutex g_mutex;
    std::unordered_map<mqtt_session_handle_t, std::unique_ptr<mqtt::MessageHandler>> g_message_handlers;
    std::unordered_map<mqtt_session_handle_t, std::unique_ptr<mqtt::SessionHandler>> g_session_handlers;

    class SessionCallbackHandler : public mqtt::SessionHandler
    {
    public:
        SessionCallbackHandler(mqtt_state_callback_t state_cb,
                               mqtt_error_callback_t error_cb,
                               void *context)
            : state_cb_(state_cb), error_cb_(error_cb), context_(context) {}

        void onStateChange(mqtt::SessionState newState) override
        {
            if (state_cb_)
            {
                state_cb_(static_cast<mqtt_session_state_t>(newState), context_);
            }
        }

        void onError(int errorCode, const char *message) override
        {
            if (error_cb_)
            {
                error_cb_(errorCode, message, context_);
            }
        }

    private:
        mqtt_state_callback_t state_cb_;
        mqtt_error_callback_t error_cb_;
        void *context_;
    };

    class MessageCallbackHandler : public mqtt::MessageHandler
    {
    public:
        MessageCallbackHandler(mqtt_message_callback_t cb, void *context)
            : cb_(cb), context_(context) {}

        void onMessage(const mqtt::Message &message) override
        {
            if (cb_)
            {
                // Message data is owned by mqtt::Message and guaranteed to exist during callback
                mqtt_message_data_t msg_data = {
                    .topic = message.getTopic(),
                    .payload = message.getPayload(),
                    .payload_length = message.getPayloadLength(),
                    .qos = static_cast<int>(message.getQoS()),
                    .retained = message.isRetained(),
                    .message_id = message.getMessageId()};

                cb_(&msg_data, context_);
            }
        }

    private:
        mqtt_message_callback_t cb_;
        void *context_;
    };
}

// Session configuration functions
int mqtt_set_int_parameter(mqtt_session_handle_t session, mqtt_parameter_t param, int32_t value)
{
    if (!session || !session->session)
        return -1;
    session->session->getConfig().set(
        static_cast<mqtt::ConnectionConfig::Parameter>(param), value);
    return 0;
}

int mqtt_set_bool_parameter(mqtt_session_handle_t session, mqtt_parameter_t param, int value)
{
    if (!session || !session->session)
        return -1;
    session->session->getConfig().set(
        static_cast<mqtt::ConnectionConfig::Parameter>(param), value != 0);
    return 0;
}

int mqtt_set_broker(mqtt_session_handle_t session, const char *url, uint16_t port)
{
    if (!session || !session->session)
        return -1;
    session->session->getConfig().setBroker(url, port);
    return 0;
}

int mqtt_set_credentials(mqtt_session_handle_t session, const char *username, const char *password)
{
    if (!session || !session->session)
        return -1;
    session->session->getConfig().setCredentials(username, password);
    return 0;
}

int mqtt_set_tls_certificates(mqtt_session_handle_t session, const char *ca_file,
                              const char *cert_file, const char *key_file)
{
    if (!session || !session->session)
        return -1;
    session->session->getConfig().setTlsCertificates(ca_file, cert_file, key_file);
    return 0;
}

// Session lifecycle functions
int mqtt_initialize(const char *app_name, const char *app_version, int debug, const char *log_file)
{
    std::lock_guard<std::mutex> lock(g_mutex);
    if (!g_factory)
    {
        g_factory = mqtt::APIFactory::getInstance();
        return g_factory->initialize(app_name, app_version, debug != 0, log_file);
    }
    return 0;
}

int mqtt_uninitialize(void)
{
    std::lock_guard<std::mutex> lock(g_mutex);
    if (g_factory)
    {
        int result = g_factory->uninitialize();
        g_factory = nullptr;
        return result;
    }
    return 0;
}

mqtt_session_handle_t mqtt_create_session(const char *client_id,
                                          mqtt_message_callback_t message_cb,
                                          mqtt_state_callback_t state_cb,
                                          mqtt_error_callback_t error_cb,
                                          void *user_context)
{
    std::lock_guard<std::mutex> lock(g_mutex);
    if (!g_factory)
        return nullptr;

    auto session_handler = std::make_unique<SessionCallbackHandler>(state_cb, error_cb, user_context);
    auto &session = g_factory->createSession(client_id, *session_handler);

    auto mqtt_session = new mqtt_session_t{
        &session,
        message_cb,
        state_cb,
        error_cb,
        user_context};

    if (message_cb)
    {
        auto message_handler = std::make_unique<MessageCallbackHandler>(message_cb, user_context);
        session.setMessageHandler(message_handler.get());
        g_message_handlers[mqtt_session] = std::move(message_handler);
    }

    g_session_handlers[mqtt_session] = std::move(session_handler);
    return mqtt_session;
}

void mqtt_destroy_session(mqtt_session_handle_t session)
{
    if (!session)
        return;
    std::lock_guard<std::mutex> lock(g_mutex);

    if (session->session)
    {
        session->session->stop(); // Ensure session is stopped before cleanup
    }

    g_message_handlers.erase(session);
    g_session_handlers.erase(session);

    if (g_factory && session->session)
    {
        g_factory->destroySession(*session->session);
    }

    delete session;
}

// Session control functions
mqtt_session_state_t mqtt_session_get_state(mqtt_session_handle_t session)
{
    if (!session || !session->session)
    {
        return MQTT_STATE_DISCONNECTED;
    }
    return static_cast<mqtt_session_state_t>(session->session->getState());
}

int mqtt_session_start(mqtt_session_handle_t session)
{
    if (!session || !session->session)
        return -1;
    return session->session->start() ? 0 : -1;
}

int mqtt_session_stop(mqtt_session_handle_t session)
{
    if (!session || !session->session)
        return -1;
    return session->session->stop() ? 0 : -1;
}

// Subscription functions
int64_t mqtt_subscribe(mqtt_session_handle_t session, const char *topic, mqtt_qos_t qos)
{
    if (!session || !session->session)
        return -1;
    return session->session->subscribe(topic, static_cast<mqtt::Message::QoS>(qos));
}

int mqtt_unsubscribe(mqtt_session_handle_t session, int64_t handle)
{
    if (!session || !session->session)
        return -1;
    return session->session->unsubscribe(handle) ? 0 : -1;
}

// Publishing functions
int64_t mqtt_publish(mqtt_session_handle_t session, const char *topic,
                     const uint8_t *payload, size_t length,
                     mqtt_qos_t qos, int retain)
{
    if (!session || !session->session)
        return -1;
    return session->session->publish(
        topic,
        payload,
        length,
        static_cast<mqtt::Message::QoS>(qos),
        retain != 0);
}