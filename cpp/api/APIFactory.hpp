#ifndef MQTT_API_FACTORY_HPP_
#define MQTT_API_FACTORY_HPP_

#include <string>
#include "Session.hpp"
#include "dllexport.h"

namespace mqtt
{

    class APIFactory
    {
    public:
        MQTT_DLLEXPORT static APIFactory *getInstance();

        MQTT_DLLEXPORT Session &createSession(const char *clientId,
                                              SessionHandler &handler);

        MQTT_DLLEXPORT void destroySession(Session &session);

        MQTT_DLLEXPORT int initialize(const char *appName,
                                      const char *appVersion,
                                      bool debug = false,
                                      const char *logFile = nullptr);

        MQTT_DLLEXPORT int uninitialize();

    private:
        APIFactory();
        ~APIFactory();
        APIFactory(const APIFactory &) = delete;
        APIFactory &operator=(const APIFactory &) = delete;

        static APIFactory *instance_;
        static int refCount_;
    };

} // namespace mqtt
#endif // MQTT_API_FACTORY_HPP_