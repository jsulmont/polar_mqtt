#ifndef MQTT_MESSAGE_HPP_
#define MQTT_MESSAGE_HPP_

#include <string>
#include <cstdint>
#include <vector>
#include "dllexport.h"

namespace mqtt
{

    class Message
    {
    public:
        enum class QoS : int32_t
        {
            AT_MOST_ONCE = 0,
            AT_LEAST_ONCE = 1,
            EXACTLY_ONCE = 2
        };

        MQTT_DLLEXPORT const char *getTopic() const;
        MQTT_DLLEXPORT const uint8_t *getPayload() const;
        MQTT_DLLEXPORT size_t getPayloadLength() const;
        MQTT_DLLEXPORT QoS getQoS() const;
        MQTT_DLLEXPORT bool isRetained() const;
        MQTT_DLLEXPORT int64_t getMessageId() const;

    private:
        friend class MessageImpl;
        friend class Session;
        Message();
        ~Message();
        struct Impl;
        Impl *impl_;
    };

} // namespace mqtt
#endif // MQTT_MESSAGE_HPP_