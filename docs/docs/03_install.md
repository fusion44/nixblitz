---
slug: /system-installation
sidebar_position: 3
sidebar_label: System Installation
---

# System Installation

## Download the ISO image

Version v0.1.0: https://zipline.f44.fyi/u/250606_NIXBLITZ-x86_64-linux_0.1.0.iso

import AsciinemaPlayerManual from '@site/src/components/AsciinemaPlayerManual';
import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

## Run the installer

Use the USB thumb drive installer for installation on a bare metal system. Attach the ISO image to the VM to install it.

<Tabs groupId="system-type">
  <TabItem value="bare_metal" label="USB" default>

    ### Flash the ISO image

    The thumb drive must have at least 16GB of storage.

    <Tabs groupId="operating-system">
      <TabItem value="linux" label="Linux" default>
        #### Using Etcher
        Download [Balena Etcher](https://www.balena.io/etcher/) and use it to flash the ISO image to the thumb drive.

        #### Using dd
        Run the following command in a terminal:

        ```bash
        sudo dd if=250606_NIXBLITZ-x86_64-linux_0.1.0.iso of=/dev/sdX bs=4M status=progress
        ```

        Replace `/dev/sdX` with the path to the thumb drive. Insert the thumb drive into the device and turn the power on.
      </TabItem>
      <TabItem value="macos" label="MacOS">
        TODO
      </TabItem>
      <TabItem value="windows" label="Windows">
        #### Using Etcher
        Download [Balena Etcher](https://www.balena.io/etcher/) and use it to flash the ISO image to the thumb drive.

        Insert the thumb drive into the device and turn the power on.
      </TabItem>
    </Tabs>

  </TabItem>
  <TabItem value="proxmox" label="Proxmox" default>
    TODO
  </TabItem>

  <TabItem value="qemu" label="QEMU" default>
    <Tabs groupId="shell-type">
      <TabItem value="bash" label="bash" default>
          ```bash
              qemu-system-x86_64 -enable-kvm -m 16384 -smp 4 \
                -netdev user,id=mynet0,hostfwd=tcp::10022-:22,hostfwd=tcp::8080-:8080 \
                -device virtio-net-pci,netdev=mynet0 \
                -drive file=nixblitz-disk.qcow2,if=none,id=virtio0,format=qcow2 \
                -device virtio-blk-pci,drive=virtio0 \
                -cdrom FILE_NAME.iso
          ```
          Replace `FILE_NAME.iso` with the path to the ISO image.
      </TabItem>
      <TabItem value="nushell" label="nu" default>
          ```nushell
              (qemu-system-x86_64 -enable-kvm -m 16384 -smp 4
                -netdev user,id=mynet0,hostfwd=tcp::10022-:22,hostfwd=tcp::8080-:8080
                -device virtio-net-pci,netdev=mynet0
                -drive file=nixblitz-disk.qcow2,if=none,id=virtio0,format=qcow2
                -device virtio-blk-pci,drive=virtio0
                -cdrom FILE_NAME.iso)
          ```
          Replace `FILE_NAME.iso` with the path to the ISO image.
      </TabItem>
    </Tabs>
  </TabItem>
</Tabs>

## Boot the nixblitz

When successful, the nixblitz installer splash screen is shown:

<div class="image-container">
    <img src="img/install/010_boot_menu.png" alt="nixblitz installer boot menu" width="600"/>
    <div class="image-label">The installer boot menu</div>
</div>
Press the `Enter` key to boot into the installer or let the counter count down.
The system will not boot into the installer NixOS system.

:::info
At this point SSH daemon is running and can be used to log into the device and run the installer from there:

```bash
# with default port
ssh -oUserKnownHostsFile=/dev/null -oStrictHostKeyChecking=no nixos@192.168.0.101

# with custom port (when running in a VM, etc)
ssh -oUserKnownHostsFile=/dev/null -oStrictHostKeyChecking=no nixos@192.168.0.101 -p 2222
```

Replace the IP with the IP of the device.

The default password is `nixblitz`
:::

After logging in or opening the webbrowser at `http://localhost:8080/install` the welcome will be shown:

<div class="image-container">
    <img src="img/install/020_welcome_screen.png" alt="left: SSH welcome screen right: web based welcome screen" width="900"/>
    <div class="image-label">left: SSH welcome screen right: web based welcome screen</div>
</div>

Follow the instructions to go to the next screen. The installer will try to detect the hardware it's running on and show some info about it:

<div class="image-container">
    <img src="img/install/030_system_check.png" alt="system check screenshot" width="900"/>
    <div class="image-label">Result of the system check</div>
</div>
Note, this currently lacks a lot of important checks.

The next step is to configure the system. The installer will show the nixblitz CLI to change any settings. Access the WebApp to change the settings.

:::info
Do not use both at once during configuration: changes are not synchronized between the two components, unless the WebApp and the TUI are manually refreshed.
:::

<div class="image-container">
    <img src="img/install/040_sys_config.png" alt="screenshot of the configuration screen" width="900"/>
    <div class="image-label">Configuration screen</div>
</div>

<Tabs groupId="settings-app">
  <TabItem value="cli" label="CLI" default>
    <img src="img/install/050_cli.png" alt="nixblitz installer boot menu" width="800"/>
    **Navigating the CLI:**

    - The currently focused pane is highlighted in red.
    - `left right` or `hl` => navigate between panes.
    - `up down ` or `jk` => navigate app and option list entries.
    - `enter` => change an option; booleans will just toggle, others will open a popup.
    - `tab` => switch elements in popups.
    - `q` or `ctrl-c` => quit the app.

  </TabItem>
  <TabItem value="webapp" label="WebApp" default>
    The WebApp is a graphical user interface for the Nixblitz CLI. It shows the same options as the TUI.
    <img src="img/install/050_webapp.png" alt="nixblitz webapp" width="800"/>
    The web app is served on port 80.
    All changes are saved automatically.
  </TabItem>
</Tabs>

:::info

**Installing vs Enabling Services**

In traditional Linux systems, the term "installing" is used to install software or services on systems like Ubuntu or Arch Linux. In nixblitz, we use the term "enabling" instead of "installing" because it makes more sense in the context of NixOS.
:::

Next, the installer will ask you to select the disk to install nixblitz onto.

<div class="image-container">
    <img src="img/install/041_select_disk.png" alt="screenshot of the disk selection screen" width="900"/>
    <div class="image-label">Disk selection</div>
</div>

:::info

- Currently, nixblitz only supports one disk only.
- It is possible to install on a 1TB disk for testing, but it is not recommended. It is planned to support multiple disks in the future.
  :::

## Installation

The system is now ready to install.

<div class="image-container">
    <img src="img/install/060_pre_install_confirm.png" alt="screenshot of the pre install confirmation screen" width="900"/>
    <div class="image-label">Starting the installation</div>
</div>

Confirm the dialog to start the installation. This process will take a few minutes, depending on your system. The system will be built with your configuration and then installed.

<div class="image-container">
    <img src="img/install/060_installation.png" alt="installing the system" width="900"/>
    <div class="image-label">Installing the system</div>
</div>
The installation is now complete and there are a few options available:

<div class="image-container">
    <img src="img/install/070_post_install.png" alt="nixblitz post install menu" width="900"/>
    <div class="image-label">Post-installation screen</div>
</div>

You can now reboot the system and log in via SSH.
