# FindPahoMQTTC.cmake

find_path(PahoMQTTC_INCLUDE_DIR
    NAMES MQTTClient.h
    PATHS
    /opt/homebrew/include
    /usr/local/include
    /usr/include
)

find_library(PahoMQTTC_LIBRARY
    NAMES
    paho-mqtt3c
    libpaho-mqtt3c
    PATHS
    /opt/homebrew/lib
    /usr/local/lib
    /usr/lib
)

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(PahoMQTTC
    REQUIRED_VARS
    PahoMQTTC_LIBRARY
    PahoMQTTC_INCLUDE_DIR
)

if(PahoMQTTC_FOUND)
    set(PahoMQTTC_LIBRARIES ${PahoMQTTC_LIBRARY})
    set(PahoMQTTC_INCLUDE_DIRS ${PahoMQTTC_INCLUDE_DIR})
endif()

mark_as_advanced(PahoMQTTC_INCLUDE_DIR PahoMQTTC_LIBRARY)

# Debug output
message(STATUS "PahoMQTTC_FOUND: ${PahoMQTTC_FOUND}")
message(STATUS "PahoMQTTC_INCLUDE_DIRS: ${PahoMQTTC_INCLUDE_DIRS}")
message(STATUS "PahoMQTTC_LIBRARIES: ${PahoMQTTC_LIBRARIES}")