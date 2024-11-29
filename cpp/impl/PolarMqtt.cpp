#include "PolarMqtt.hpp"
#include <MQTTClient.h>
#include <map>
#include <mutex>
#include <vector>
#include <iostream>

namespace mqtt
{

    // Message Implementation
    struct Message::Impl
    {
        std::string topic;
        std::vector<uint8_t> payload;
        Message::QoS qos{Message::QoS::AT_MOST_ONCE};
        bool retained{false};
        int64_t messageId{0};
    };

    Message::Message() : impl_(new Impl()) {}
    Message::~Message() { delete impl_; }

    const char *Message::getTopic() const { return impl_->topic.c_str(); }
    const uint8_t *Message::getPayload() const { return impl_->payload.data(); }
    size_t Message::getPayloadLength() const { return impl_->payload.size(); }
    Message::QoS Message::getQoS() const { return impl_->qos; }
    bool Message::isRetained() const { return impl_->retained; }
    int64_t Message::getMessageId() const { return impl_->messageId; }

    // Connection Config Implementation
    struct ConnectionConfig::Impl
    {
        std::string broker;
        uint16_t port;
        std::string username;
        std::string password;
        std::string caFile;
        std::string certFile;
        std::string keyFile;
        int32_t keepAliveInterval{60};
        bool cleanSession{true};
        int32_t connectionTimeout{30};
        int32_t maxInflight{10};
        int32_t maxQueuedMessages{100};
        int32_t reconnectDelay{5};
        bool tlsEnabled{false};
    };

    ConnectionConfig::ConnectionConfig() : impl_(new Impl()) {}
    ConnectionConfig::~ConnectionConfig() { delete impl_; }

    ConnectionConfig &ConnectionConfig::set(Parameter param, int32_t value)
    {
        switch (param)
        {
        case Parameter::KEEP_ALIVE_INTERVAL:
            impl_->keepAliveInterval = value;
            break;
        case Parameter::CONNECTION_TIMEOUT:
            impl_->connectionTimeout = value;
            break;
        case Parameter::MAX_INFLIGHT:
            impl_->maxInflight = value;
            break;
        case Parameter::MAX_QUEUED_MESSAGES:
            impl_->maxQueuedMessages = value;
            break;
        case Parameter::RECONNECT_DELAY:
            impl_->reconnectDelay = value;
            break;
        default:
            break;
        }
        return *this;
    }

    ConnectionConfig &ConnectionConfig::set(Parameter param, bool value)
    {
        switch (param)
        {
        case Parameter::CLEAN_SESSION:
            impl_->cleanSession = value;
            break;
        case Parameter::TLS_ENABLED:
            impl_->tlsEnabled = value;
            break;
        default:
            break;
        }
        return *this;
    }

    ConnectionConfig &ConnectionConfig::setBroker(const char *url, uint16_t port)
    {
        impl_->broker = url ? url : "";
        impl_->port = port;
        return *this;
    }

    ConnectionConfig &ConnectionConfig::setCredentials(const char *username, const char *password)
    {
        impl_->username = username ? username : "";
        impl_->password = password ? password : "";
        return *this;
    }

    ConnectionConfig &ConnectionConfig::setTlsCertificates(const char *caFile,
                                                           const char *certFile,
                                                           const char *keyFile)
    {
        impl_->caFile = caFile ? caFile : "";
        impl_->certFile = certFile ? certFile : "";
        impl_->keyFile = keyFile ? keyFile : "";
        impl_->tlsEnabled = true;
        return *this;
    }

    // Session Implementation
    struct Session::Impl
    {
        MQTTClient client;
        std::string clientId;
        ConnectionConfig config;
        MessageHandler *msgHandler{nullptr};
        SessionHandler *sessionHandler;
        SessionState currentState{SessionState::DISCONNECTED};
        std::mutex stateMutex;
        std::map<int64_t, std::string> subscriptions;
        int64_t nextSubHandle{1};
        int64_t nextMessageId{1};

        static int onMessageCallback(void *context, char *topicName, int topicLen,
                                     MQTTClient_message *message)
        {
            auto *impl = static_cast<Session::Impl *>(context);
            if (impl->msgHandler)
            {
                Message msg;
                msg.impl_->topic = topicName ? std::string(topicName) : std::string();
                msg.impl_->payload.assign(
                    static_cast<uint8_t *>(message->payload),
                    static_cast<uint8_t *>(message->payload) + message->payloadlen);
                msg.impl_->qos = static_cast<Message::QoS>(message->qos);
                msg.impl_->retained = message->retained;
                msg.impl_->messageId = message->msgid;

                impl->msgHandler->onMessage(msg);
            }
            MQTTClient_freeMessage(&message);
            MQTTClient_free(topicName);
            return 1;
        }

        static void onConnectionLost(void *context, char *cause)
        {
            auto *impl = static_cast<Session::Impl *>(context);
            {
                std::lock_guard<std::mutex> lock(impl->stateMutex);
                impl->currentState = SessionState::RECONNECTING;
            }
            if (impl->sessionHandler)
            {
                impl->sessionHandler->onStateChange(SessionState::RECONNECTING);
                impl->sessionHandler->onError(-1, cause ? cause : "Connection lost");
            }
        }
    };

    Session::Session(const char *clientId, SessionHandler &handler)
        : impl_(new Impl())
    {
        impl_->clientId = clientId;
        impl_->sessionHandler = &handler;
        impl_->currentState = SessionState::DISCONNECTED;
    }

    Session::~Session()
    {
        stop();
        delete impl_;
    }

    Session::State Session::getState() const
    {
        std::lock_guard<std::mutex> lock(impl_->stateMutex);
        return static_cast<State>(impl_->currentState);
    }

    ConnectionConfig &Session::getConfig()
    {
        return impl_->config;
    }

    bool Session::start()
    {
        if (!impl_ || !impl_->config.impl_)
        {
            std::cerr << "Invalid implementation pointers" << std::endl;
            return false;
        }

        MQTTClient_connectOptions conn_opts = MQTTClient_connectOptions_initializer;
        auto &cfg = impl_->config.impl_;

        if (cfg->broker.empty())
        {
            std::cerr << "Broker URL not set" << std::endl;
            return false;
        }

        std::string serverURI = (cfg->tlsEnabled ? "ssl://" : "tcp://") +
                                cfg->broker + ":" + std::to_string(cfg->port);

        int rc = MQTTClient_create(&impl_->client, serverURI.c_str(),
                                   impl_->clientId.c_str(),
                                   MQTTCLIENT_PERSISTENCE_NONE, nullptr);
        if (rc != MQTTCLIENT_SUCCESS)
        {
            if (impl_->sessionHandler)
            {
                impl_->sessionHandler->onError(rc, "Failed to create client");
            }
            return false;
        }

        MQTTClient_setCallbacks(impl_->client, impl_,
                                Impl::onConnectionLost,
                                Impl::onMessageCallback,
                                nullptr);

        conn_opts.keepAliveInterval = cfg->keepAliveInterval;
        conn_opts.cleansession = cfg->cleanSession;
        conn_opts.retryInterval = cfg->reconnectDelay;
        conn_opts.reliable = 1;

        if (!cfg->username.empty())
        {
            conn_opts.username = cfg->username.c_str();
            conn_opts.password = cfg->password.c_str();
        }

        if (cfg->tlsEnabled)
        {
            MQTTClient_SSLOptions ssl_opts = MQTTClient_SSLOptions_initializer;
            ssl_opts.trustStore = cfg->caFile.c_str();
            ssl_opts.keyStore = cfg->certFile.c_str();
            ssl_opts.privateKey = cfg->keyFile.c_str();
            conn_opts.ssl = &ssl_opts;
        }

        {
            std::lock_guard<std::mutex> lock(impl_->stateMutex);
            impl_->currentState = SessionState::CONNECTING;
        }

        rc = MQTTClient_connect(impl_->client, &conn_opts);
        if (rc != MQTTCLIENT_SUCCESS)
        {
            if (impl_->sessionHandler)
            {
                impl_->sessionHandler->onError(rc, "Connection failed");
            }
            {
                std::lock_guard<std::mutex> lock(impl_->stateMutex);
                impl_->currentState = SessionState::DISCONNECTED;
            }
            return false;
        }

        {
            std::lock_guard<std::mutex> lock(impl_->stateMutex);
            impl_->currentState = SessionState::CONNECTED;
        }

        if (impl_->sessionHandler)
        {
            impl_->sessionHandler->onStateChange(SessionState::CONNECTED);
        }

        return true;
    }

    bool Session::stop()
    {
        if (impl_->client)
        {
            MQTTClient_disconnect(impl_->client, 10000);
            MQTTClient_destroy(&impl_->client);
            {
                std::lock_guard<std::mutex> lock(impl_->stateMutex);
                impl_->currentState = SessionState::DISCONNECTED;
            }
            if (impl_->sessionHandler)
            {
                impl_->sessionHandler->onStateChange(SessionState::DISCONNECTED);
            }
        }
        return true;
    }

    int64_t Session::subscribe(const char *topic, Message::QoS qos)
    {
        int rc = MQTTClient_subscribe(impl_->client, topic, static_cast<int>(qos));
        if (rc != MQTTCLIENT_SUCCESS)
        {
            if (impl_->sessionHandler)
            {
                impl_->sessionHandler->onError(rc, "Subscribe failed");
            }
            return -1;
        }

        int64_t handle = impl_->nextSubHandle++;
        impl_->subscriptions[handle] = topic;
        return handle;
    }

    bool Session::unsubscribe(int64_t handle)
    {
        auto it = impl_->subscriptions.find(handle);
        if (it == impl_->subscriptions.end())
        {
            return false;
        }

        int rc = MQTTClient_unsubscribe(impl_->client, it->second.c_str());
        if (rc != MQTTCLIENT_SUCCESS)
        {
            if (impl_->sessionHandler)
            {
                impl_->sessionHandler->onError(rc, "Unsubscribe failed");
            }
            return false;
        }

        impl_->subscriptions.erase(it);
        return true;
    }

    int64_t Session::publish(const char *topic, const uint8_t *payload,
                             size_t length, Message::QoS qos, bool retain)
    {
        MQTTClient_message pubmsg = MQTTClient_message_initializer;
        pubmsg.payload = const_cast<uint8_t *>(payload);
        pubmsg.payloadlen = static_cast<int>(length);
        pubmsg.qos = static_cast<int>(qos);
        pubmsg.retained = retain;

        int64_t messageId = impl_->nextMessageId++;
        pubmsg.msgid = static_cast<int>(messageId);

        int rc = MQTTClient_publishMessage(impl_->client, topic, &pubmsg, nullptr);
        if (rc != MQTTCLIENT_SUCCESS)
        {
            if (impl_->sessionHandler)
            {
                impl_->sessionHandler->onError(rc, "Publish failed");
            }
            return -1;
        }

        return messageId;
    }

    void Session::setMessageHandler(MessageHandler *handler)
    {
        impl_->msgHandler = handler;
    }

    void Session::setSessionHandler(SessionHandler *handler)
    {
        impl_->sessionHandler = handler;
    }

    // APIFactory Implementation
    APIFactory *APIFactory::instance_ = nullptr;
    int APIFactory::refCount_ = 0;

    APIFactory::APIFactory() {}
    APIFactory::~APIFactory() {}

    APIFactory *APIFactory::getInstance()
    {
        if (!instance_)
        {
            instance_ = new APIFactory();
        }
        ++refCount_;
        return instance_;
    }

    int APIFactory::initialize(const char *appName, const char *appVersion,
                               bool debug, const char *logFile)
    {
        return 0;
    }

    int APIFactory::uninitialize()
    {
        --refCount_;
        if (refCount_ == 0 && instance_)
        {
            delete instance_;
            instance_ = nullptr;
        }
        return 0;
    }

    Session &APIFactory::createSession(const char *clientId, SessionHandler &handler)
    {
        return *(new Session(clientId, handler));
    }

    void APIFactory::destroySession(Session &session)
    {
        delete &session;
    }

} // namespace mqtt