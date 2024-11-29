#ifndef MQTT_SESSION_HPP_
#define MQTT_SESSION_HPP_

#include <string>
#include "Message.hpp"
#include "MessageHandler.hpp"
#include "SessionHandler.hpp"
#include "ConnectionConfig.hpp"
#include "dllexport.h"

namespace mqtt
{
    class Session
    {
    public:
        using State = SessionState; 

        MQTT_DLLEXPORT State getState() const;
        MQTT_DLLEXPORT ConnectionConfig &getConfig();
        MQTT_DLLEXPORT bool start();
        MQTT_DLLEXPORT bool stop();

        // Subscription management
        MQTT_DLLEXPORT int64_t subscribe(const char *topic, Message::QoS qos);
        MQTT_DLLEXPORT bool unsubscribe(int64_t handle);

        // Publishing
        MQTT_DLLEXPORT int64_t publish(const char *topic,
                                       const uint8_t *payload,
                                       size_t length,
                                       Message::QoS qos = Message::QoS::AT_MOST_ONCE,
                                       bool retain = false);

        // Handler registration
        MQTT_DLLEXPORT void setMessageHandler(MessageHandler *handler);
        MQTT_DLLEXPORT void setSessionHandler(SessionHandler *handler);

    private:
        friend class APIFactory;
        Session();
        Session(const char *clientId, SessionHandler &handler);
        ~Session();
        struct Impl;
        Impl *impl_;
    };

} // namespace mqtt
#endif // MQTT_SESSION_HPP_