#ifndef MQTT_CONNECTION_CONFIG_HPP_
#define MQTT_CONNECTION_CONFIG_HPP_

#include <string>
#include "dllexport.h"

namespace mqtt
{

    class ConnectionConfig
    {
    public:
        enum class Parameter : int32_t
        {
            KEEP_ALIVE_INTERVAL = 0,
            CLEAN_SESSION = 1,
            CONNECTION_TIMEOUT = 2,
            MAX_INFLIGHT = 3,
            MAX_QUEUED_MESSAGES = 4,
            RECONNECT_DELAY = 5,
            TLS_ENABLED = 6
        };

        MQTT_DLLEXPORT ConnectionConfig &set(Parameter param, int32_t value);
        MQTT_DLLEXPORT ConnectionConfig &set(Parameter param, bool value);
        MQTT_DLLEXPORT ConnectionConfig &setBroker(const char *url, uint16_t port);
        MQTT_DLLEXPORT ConnectionConfig &setCredentials(const char *username, const char *password);
        MQTT_DLLEXPORT ConnectionConfig &setTlsCertificates(const char *caFile, const char *certFile, const char *keyFile);

    private:
        friend class Session;
        ConnectionConfig();
        ~ConnectionConfig();
        struct Impl;
        Impl *impl_;
    };

} // namespace mqtt
#endif // MQTT_CONNECTION_CONFIG_HPP_