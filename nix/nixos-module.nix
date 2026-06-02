# NixOS module: grant non-root libusb access to the SpaceMouse Enterprise LCD.
#
# The LCD lives on the driverless vendor interface (USB 256f:c633, interface 0).
# A plain `SUBSYSTEM=="hidraw"` rule only covers the HID interface spacenavd uses,
# not the USB device node libusb opens — so the daemon needs this `SUBSYSTEM=="usb"`
# rule to run without sudo, alongside spacenavd.
{
  lib,
  config,
  ...
}: let
  cfg = config.hardware.spaceMouseEnterpriseLcd;
in {
  options.hardware.spaceMouseEnterpriseLcd = {
    enable =
      lib.mkEnableOption "non-root LCD access for the SpaceMouse Enterprise";
  };

  config = lib.mkIf cfg.enable {
    services.udev.extraRules = ''
      # SpaceMouse Enterprise LCD — vendor interface 0, libusb access for the active user
      SUBSYSTEM=="usb", ATTRS{idVendor}=="256f", ATTRS{idProduct}=="c633", TAG+="uaccess"
    '';
  };
}
