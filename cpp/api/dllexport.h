#ifndef MQTT_DLLEXPORT_H_
#define MQTT_DLLEXPORT_H_

#ifdef MQTT_EXPORTS
#define MQTT_DLLEXPORT __declspec(dllexport)
#else
#define MQTT_DLLEXPORT
#endif

#endif // MQTT_DLLEXPORT_H_
