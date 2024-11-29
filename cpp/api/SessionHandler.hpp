#ifndef MQTT_SESSION_HANDLER_HPP_
#define MQTT_SESSION_HANDLER_HPP_

namespace mqtt
{
    // Forward declare the State enum
    class Session;
    enum class SessionState : int32_t
    {
        DISCONNECTED = 0,
        CONNECTING = 1,
        CONNECTED = 2,
        RECONNECTING = 3
    };

    class SessionHandler
    {
    public:
        virtual ~SessionHandler() {}
        virtual void onStateChange(SessionState newState) = 0;
        virtual void onError(int errorCode, const char *message) = 0;
    };

} // namespace mqtt
#endif // MQTT_SESSION_HANDLER_HPP_