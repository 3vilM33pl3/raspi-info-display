#!/bin/bash
# Test script for TCA9548A I2C Multiplexer

echo "TCA9548A I2C Multiplexer Test Script"
echo "===================================="
echo

# Check if i2c-tools is installed
if ! command -v i2cdetect &> /dev/null; then
    echo "Error: i2c-tools is not installed. Run: sudo apt install i2c-tools"
    exit 1
fi

# Default I2C bus and multiplexer address
I2C_BUS=${1:-1}
MUX_ADDR=${2:-0x70}

echo "Using I2C bus: $I2C_BUS"
echo "Multiplexer address: $MUX_ADDR"
echo

# Detect multiplexer
echo "Scanning for TCA9548A at address $MUX_ADDR..."
if i2cdetect -y $I2C_BUS $(printf "%d" $MUX_ADDR) $(printf "%d" $MUX_ADDR) 2>/dev/null | grep -q "70"; then
    echo "✓ TCA9548A detected at address $MUX_ADDR"
else
    echo "✗ TCA9548A not found at address $MUX_ADDR"
    echo "  Scanning entire I2C bus $I2C_BUS..."
    i2cdetect -y $I2C_BUS
    exit 1
fi

echo
echo "Testing multiplexer channels..."
echo "-------------------------------"

# Test each channel
for channel in {0..7}; do
    echo -n "Channel $channel: "
    
    # Select channel on multiplexer
    channel_mask=$((1 << channel))
    if i2cset -y $I2C_BUS $MUX_ADDR $(printf "0x%02x" $channel_mask) 2>/dev/null; then
        # Scan for devices on this channel
        devices=$(i2cdetect -y $I2C_BUS 0x03 0x77 2>/dev/null | grep -E "^[0-9]0:" | grep -oE "[0-9a-f]{2}" | grep -v "^[037][0-9a-f]$" | grep -v "^70$")
        
        if [ -n "$devices" ]; then
            echo "Devices found: $devices"
            
            # Check for common devices
            for device in $devices; do
                case $device in
                    3c|3d)
                        echo "  → Likely SSD1306 OLED display at 0x$device"
                        ;;
                    48|49|4a|4b|4c|4d|4e|4f)
                        echo "  → Likely ADS1x15 ADC at 0x$device"
                        ;;
                    20|21|22|23|24|25|26|27)
                        echo "  → Likely PCF8574 I/O expander at 0x$device"
                        ;;
                    68|69|6a|6b)
                        echo "  → Likely MPU6050/DS3231 at 0x$device"
                        ;;
                    76|77)
                        echo "  → Likely BME280/BMP280 sensor at 0x$device"
                        ;;
                    *)
                        echo "  → Unknown device at 0x$device"
                        ;;
                esac
            done
        else
            echo "No devices"
        fi
    else
        echo "Failed to select channel"
    fi
done

echo
echo "Test complete!"