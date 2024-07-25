module;

#include <libusb.h>

export module usb;

import std;

export namespace usb {

class Error : public std::runtime_error {
public:
  Error(int err_code)
      : runtime_error(libusb_error_name(err_code)), _code(err_code) {}
  int code() const { return _code; }

private:
  int _code;
};

void check(int err) {
  if (err < 0) {
    throw Error(err);
  }
}

template <typename T, void (*del)(T *)>
using Handle = std::unique_ptr<T, decltype(del)>;

using context = Handle<libusb_context, libusb_exit>;
context init() {
  libusb_context *ctx = nullptr;
  check(libusb_init(&ctx));
  return context{ctx, libusb_exit};
}

using device = Handle<libusb_device, libusb_unref_device>;
std::vector<device> get_device_list(context &ctx) {

  libusb_device **list = nullptr;
  int count = libusb_get_device_list(ctx.get(), &list);
  check(count);

  // Create device handles for all the devices
  std::vector<device> ret;
  for (int i = 0; i < count; i++)
    ret.emplace_back(device{list[i], libusb_unref_device});

  // Free the outer list, but not the handles
  libusb_free_device_list(list, false);
  return ret;
}

using device_handle = Handle<libusb_device_handle, libusb_close>;
device_handle open(device &dev) {
  libusb_device_handle *hnd = nullptr;
  int err = libusb_open(dev.get(), &hnd);
  check(err);
  return device_handle{hnd, libusb_close};
}

using device_descriptor = libusb_device_descriptor;
device_descriptor get_device_descriptor(const device &dev) {
  device_descriptor ret;
  check(libusb_get_device_descriptor(dev.get(), &ret));
  return ret;
}

device_handle open_device_with_vid_pid(context &ctx, uint16_t vid,
                                       uint16_t pid) {
  device_handle ret{libusb_open_device_with_vid_pid(ctx.get(), vid, pid),
                    libusb_close};
  if (ret == nullptr)
    throw Error(LIBUSB_ERROR_NOT_FOUND);
  return ret;
}

class Interface {
  static constexpr int Invalid = -1;
  int handle = Invalid;
  libusb_device_handle *dev = nullptr;

  int try_release() {
    if (handle != Invalid) {
      int h = handle;
      handle = Invalid;
      return libusb_release_interface(dev, h);
    }

    return 0;
  }

public:
  explicit Interface(int i, device_handle &dev) noexcept
      : handle(i), dev(dev.get()) {}

  Interface(const Interface &) = delete;
  Interface &operator=(const Interface &) = delete;

  Interface(Interface &&from) { (*this) = std::move(from); }

  void release_interface() { check(try_release()); }

  Interface &operator=(Interface &&from) {
    release_interface();
    handle = from.handle;
    dev = from.dev;
    from.handle = Invalid;
    return *this;
  }

  ~Interface() {
    if (int e; (e = libusb_release_interface(dev, handle)) != 0)
      std::cerr << "Failed to release interface: " << Error(e).what() << "\n";
  }
};

Interface claim_interface(device_handle &dev, int interface) {
  usb::check(libusb_claim_interface(dev.get(), interface));
  return Interface{interface, dev};
}

template <typename T, typename... Args>
constexpr bool one_of = (... || std::same_as<T, Args>);

template <typename T>
concept NonConstByteData =
    std::ranges::contiguous_range<T> &&
    one_of<std::ranges::range_value_t<T>, char, unsigned char, std::byte> &&
    !std::is_const_v<T>;

template <NonConstByteData Range>
int bulk_transfer(device_handle &dev, int endpoint, Range &&range,
                  std::chrono::milliseconds timeout = std::chrono::milliseconds{
                      0}) {
  using std::begin;
  using std::end;
  int sent = 0;
  int err = libusb_bulk_transfer(
      dev.get(), endpoint, reinterpret_cast<unsigned char *>(&*begin(range)),
      end(range) - begin(range), &sent, timeout.count());
  check(err);
  return sent;
}

template <NonConstByteData Range>
void control_transfer(
    device_handle &dev, std::uint8_t request_type, std::uint8_t request,
    std::uint16_t value, std::uint16_t index, Range &&range,
    std::chrono::milliseconds timeout = std::chrono::milliseconds{0}) {
  using std::begin;
  using std::end;

  int err =
      libusb_control_transfer(dev.get(), request_type, request, value, index,
                              reinterpret_cast<unsigned char *>(&*begin(range)),
                              end(range) - begin(range), timeout.count());
  check(err);
}

void reset_device(device_handle &dev) { check(libusb_reset_device(dev.get())); }

using config_descriptor =
    Handle<libusb_config_descriptor, libusb_free_config_descriptor>;
config_descriptor get_config_descriptor(device &dev, uint8_t config_index) {
  libusb_config_descriptor *desc = nullptr;
  check(libusb_get_config_descriptor(dev.get(), config_index, &desc));
  return config_descriptor{desc, libusb_free_config_descriptor};
}

using config_descriptor =
    Handle<libusb_config_descriptor, libusb_free_config_descriptor>;
config_descriptor get_config_descriptor_by_value(device &dev,
                                                 uint8_t bConfigurationValue) {
  libusb_config_descriptor *desc = nullptr;
  check(libusb_get_config_descriptor_by_value(dev.get(), bConfigurationValue,
                                              &desc));
  return config_descriptor{desc, libusb_free_config_descriptor};
}

} // namespace usb
