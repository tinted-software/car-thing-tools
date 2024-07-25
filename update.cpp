import std;
import usb;

constexpr std::uint16_t AML_ID_VENDOR = 0x1B8E;
constexpr std::uint16_t AML_ID_PRODUCT = 0xC003;

void bulkcmd(usb::device_handle &handle, std::string command) {
  std::uint8_t request = 0x34;

  usb::control_transfer(handle, 64, request, 0, 2, command,
                        std::chrono::milliseconds{1000});

  while (true) {
    std::string buffer;
    buffer.resize(512);
    usb::bulk_transfer(handle, 0x81, buffer);

    if (buffer.contains("success")) {
      break;
    }

    std::this_thread::sleep_for(std::chrono::milliseconds{3000});
  }
}

void write_large_memory(usb::device_handle &handle, std::uint32_t base_address,
                        std::istream &input_stream) {
  std::uint8_t request_type = 0x40;
  std::uint8_t request = 1;
  std::size_t i = 0;

  while (true) {
    std::array<char, 64> buffer;

    if (!input_stream.read(buffer.data(), buffer.size()))
      break;

    std::uint32_t address = base_address + i;

    std::uint16_t value = (address >> 16) & 0xFFFF;
    std::uint16_t index = address & 0xFFFF;

    usb::control_transfer(handle, request_type, request, value, index, buffer,
                          std::chrono::milliseconds{1000});

    i += 64;
  }

  while (true) {
    std::string buffer;
    buffer.resize(512);
    usb::bulk_transfer(handle, 0x81, buffer);

    if (buffer.contains("success")) {
      break;
    }

    std::this_thread::sleep_for(std::chrono::milliseconds{3000});
  }
}

auto main() -> int {
  usb::context usb_context = usb::init();

  std::vector<usb::device> devices = usb::get_device_list(usb_context);

  auto device =
      std::find_if(devices.begin(), devices.end(), [](const auto &device) {
        auto descriptor = usb::get_device_descriptor(device);

        return descriptor.idVendor =
                   AML_ID_VENDOR && descriptor.idProduct == AML_ID_PRODUCT;
      });
  if (device == devices.end()) {
    std::cerr << "No amlogic device found" << std::endl;
    return 1;
  }

  usb::device_handle handle = usb::open(*device);

  std::ifstream mainline_u_boot_image("../u-boot/u-boot.bin", std::ios::binary);
  // write_large_memory(handle, 0x01080000, mainline_u_boot_image);
  // bulkcmd(handle, "go 0x01080000");

  return 0;
}
