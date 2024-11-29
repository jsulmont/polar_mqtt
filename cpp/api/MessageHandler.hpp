#ifndef MQTT_MESSAGE_HANDLER_HPP_
#define MQTT_MESSAGE_HANDLER_HPP_

#include "Message.hpp"

namespace mqtt
{

    class MessageHandler
    {
    public:
        virtual ~MessageHandler() {}
        virtual void onMessage(const Message &message) = 0;
    };

} // namespace mqtt
#endif // MQTT_MESSAGE_HANDLER_HPP_