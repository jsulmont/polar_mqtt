cmake_minimum_required(VERSION 3.10)
project(polar-mqtt-cpp)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../cmake")

find_package(PahoMQTTC REQUIRED)

# Debug output of variables
message(STATUS "CMAKE_MODULE_PATH: ${CMAKE_MODULE_PATH}")

# C++ implementation library
add_library(polar_mqtt_impl SHARED

    "${CMAKE_CURRENT_SOURCE_DIR}/impl/PolarMqtt.cpp"
)

target_include_directories(polar_mqtt_impl
    PUBLIC
    "${CMAKE_CURRENT_SOURCE_DIR}/api"
    PRIVATE
    ${PahoMQTTC_INCLUDE_DIRS}
)

target_link_libraries(polar_mqtt_impl
    PRIVATE
    ${PahoMQTTC_LIBRARIES}
)

set_target_properties(polar_mqtt_impl PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
)

# C Bridge library
add_library(polar_mqtt_bridge SHARED
    "${CMAKE_CURRENT_SOURCE_DIR}/bridge/src/mqtt_c.cpp"
)

target_include_directories(polar_mqtt_bridge
    PUBLIC
    "${CMAKE_CURRENT_SOURCE_DIR}/bridge/include"
    PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/api"
)

target_link_libraries(polar_mqtt_bridge
    PRIVATE
    polar_mqtt_impl
)

set_target_properties(polar_mqtt_bridge PROPERTIES
    CXX_STANDARD 17
    CXX_STANDARD_REQUIRED ON
)

# Install targets
install(TARGETS polar_mqtt_impl polar_mqtt_bridge
    LIBRARY DESTINATION lib
    RUNTIME DESTINATION bin
)