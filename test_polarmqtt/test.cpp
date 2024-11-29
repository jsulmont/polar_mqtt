#include <PolarMqtt.hpp>
#include <iostream>
#include <thread>
#include <chrono>
#include <iomanip>

class TestSessionHandler : public mqtt::SessionHandler
{
public:
    void onStateChange(mqtt::SessionState newState) override
    {
        std::cout << "Session state changed to: " << static_cast<int>(newState) << std::endl;
    }

    void onError(int errorCode, const char *message) override
    {
        std::cout << "Error occurred: " << errorCode << " - " << message << std::endl;
    }
};

class TestMessageHandler : public mqtt::MessageHandler
{
public:
    void onMessage(const mqtt::Message &message) override
    {
        std::cout << "\n=== Message Received ===" << std::endl;

        std::cout << "Topic address: " << static_cast<const void *>(message.getTopic()) << std::endl;
        if (message.getTopic())
        {
            const char *topic = message.getTopic();
            size_t topicLen = 0;
            // Safely check the first few bytes of the topic
            std::cout << "First 16 bytes of topic memory: ";
            for (int i = 0; i < 16; i++)
            {
                std::cout << std::hex << std::setw(2) << std::setfill('0')
                          << static_cast<int>(static_cast<unsigned char>(topic[i])) << " ";
                if (topic[i] != '\0')
                    topicLen++;
                else
                    break;
            }
            std::cout << std::dec << std::endl;
            std::cout << "Calculated topic length: " << topicLen << std::endl;
        }

        std::cout << "Topic: '" << (message.getTopic() ? message.getTopic() : "nullptr") << "'" << std::endl;

        std::cout << "Payload length: " << message.getPayloadLength() << std::endl;
        if (message.getPayload() && message.getPayloadLength() > 0)
        {
            std::string payload(reinterpret_cast<const char *>(message.getPayload()),
                                message.getPayloadLength());
            std::cout << "Payload: '" << payload << "'" << std::endl;
        }

        std::cout << "QoS: " << static_cast<int>(message.getQoS()) << std::endl;
        std::cout << "Retained: " << message.isRetained() << std::endl;
        std::cout << "Message ID: " << message.getMessageId() << std::endl;
        std::cout << "===================" << std::endl;
    }
};

int main()
{
    try
    {
        std::cout << "Initializing API..." << std::endl;
        mqtt::APIFactory::getInstance()->initialize("TestApp", "1.0", true);

        TestSessionHandler sessionHandler;
        TestMessageHandler messageHandler;

        std::cout << "Creating session..." << std::endl;
        mqtt::Session &session = mqtt::APIFactory::getInstance()->createSession("TestClient", sessionHandler);
        session.setMessageHandler(&messageHandler);

        std::cout << "Configuring connection..." << std::endl;
        session.getConfig()
            .setBroker("broker.emqx.io", 1883)
            .set(mqtt::ConnectionConfig::Parameter::KEEP_ALIVE_INTERVAL, 60)
            .set(mqtt::ConnectionConfig::Parameter::CLEAN_SESSION, true);

        std::cout << "Starting session..." << std::endl;
        if (!session.start())
        {
            std::cerr << "Failed to start session" << std::endl;
            return 1;
        }

        // Subscribe to all topics
        std::cout << "Subscribing to PSENSE topics..." << std::endl;
        int64_t subHandle = session.subscribe("test/topic", mqtt::Message::QoS::AT_LEAST_ONCE);
        if (subHandle < 0)
        {
            std::cerr << "Failed to subscribe, handle: " << subHandle << std::endl;
            return 1;
        }
        std::this_thread::sleep_for(std::chrono::seconds(1));

        // Test publish to verify we can see our own messages
        std::cout << "\nPublishing test message..." << std::endl;
        const char *testTopic = "PSENSE/test";
        const char *testPayload = "{\"test\":true}";

        int64_t msgId = session.publish(
            testTopic,
            reinterpret_cast<const uint8_t *>(testPayload),
            strlen(testPayload),
            mqtt::Message::QoS::AT_LEAST_ONCE);

        if (msgId < 0)
        {
            std::cerr << "Failed to publish message, id: " << msgId << std::endl;
        }
        else
        {
            std::cout << "Published test message with id: " << msgId << std::endl;
        }

        std::cout << "\nListening for messages for 15 seconds..." << std::endl;
        std::cout << "Debugging info:" << std::endl;
        std::cout << "Test topic address: " << static_cast<const void *>(testTopic) << std::endl;
        std::cout << "Test topic content: '" << testTopic << "'" << std::endl;
        std::cout << "Test topic length: " << strlen(testTopic) << std::endl;

        std::this_thread::sleep_for(std::chrono::seconds(15));

        std::cout << "Cleaning up..." << std::endl;
        session.stop();
        mqtt::APIFactory::getInstance()->destroySession(session);
        mqtt::APIFactory::getInstance()->uninitialize();

        return 0;
    }
    catch (const std::exception &e)
    {
        std::cerr << "Exception caught: " << e.what() << std::endl;
        return 1;
    }
}
