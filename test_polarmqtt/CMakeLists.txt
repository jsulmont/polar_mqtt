cmake_minimum_required(VERSION 3.10)
project(test_polarmqtt)

set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})

# Use C++17
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Add the path to find the MQTT implementation library
set(CMAKE_PREFIX_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../build ${CMAKE_PREFIX_PATH})

# Find PahoMQTT
set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../cmake ${CMAKE_MODULE_PATH})
find_package(PahoMQTTC REQUIRED)

# Locate Cargo build directory - adjust the relative path as needed
file(GLOB_RECURSE CARGO_BUILD_DIR
    "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug/build/polar-mqtt-*/out/build/libpolar_mqtt_impl.dylib"
)
get_filename_component(CARGO_LIB_DIR "${CARGO_BUILD_DIR}" DIRECTORY)

message(STATUS "Cargo build directory: ${CARGO_LIB_DIR}")

# Find the implementation library
find_library(POLAR_MQTT_IMPL
    NAMES polar_mqtt_impl
    PATHS ${CARGO_LIB_DIR}
    NO_DEFAULT_PATH
    REQUIRED
)

# Add the test executable
add_executable(test_polarmqtt test.cpp)

# Add the include directories for the MQTT API headers
target_include_directories(test_polarmqtt PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../cpp/api
    ${PahoMQTTC_INCLUDE_DIRS}
)

# Link against the implementation library and Paho MQTT
target_link_libraries(test_polarmqtt
    PRIVATE
    ${POLAR_MQTT_IMPL}
    ${PahoMQTTC_LIBRARIES}
)

# Copy the required libraries to the output directory
if(APPLE)
    add_custom_command(TARGET test_polarmqtt POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${POLAR_MQTT_IMPL}
        $<TARGET_FILE_DIR:test_polarmqtt>
    )
endif()

# Print some debug information
message(STATUS "POLAR_MQTT_IMPL: ${POLAR_MQTT_IMPL}")
message(STATUS "PahoMQTTC_LIBRARIES: ${PahoMQTTC_LIBRARIES}")
