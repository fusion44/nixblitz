# System Installation

## Download the ISO image

Version v0.1.0: https://zipline.f44.fyi/u/250606_NIXBLITZ-x86_64-linux_0.1.0.iso

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

<img src="img/install/010_boot_menu.png" alt="nixblitz installer boot menu" width="600"/>
Press the `Enter` key to boot into the installer or let the counter count down.
The system will not boot into the installer NixOS system.

Next, the install wizard will be shown:
<img src="img/install/020_select_install_method.png" alt="nixblitz installer install method" width="600"/>
Choose `On this machine (local install)` and hit Enter. Remote installation is not yet implemented.

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

Next, the installer will try to detect the hardware it's running on and show which platform was detected:
<img src="img/install/030_detect_platform.png" alt="nixblitz installer boot menu" width="600"/>
In this case, it's running in a virtual machine. Press 'Enter' to confirm.

<details>
    <summary>Wrong platform detected?</summary>

    If the installer fails to detect the hardware correctly, the platform must be manually selected by pressing 'N' and then choosing the correct platform for the hardware.

    <img src="img/install/031_select_custom_platform.png" alt="nixblitz Installer Boot Menu" width="600"/>

</details>

Next, the installer will ask you to select the disk to install nixblitz onto.

<img src="img/install/040_select_install_disk.png" alt="nixblitz installer boot menu" width="600"/>
Press `Enter` to confirm.

The next step is to configure the system. The installer will show the nixblitz CLI to change any settings. Access the WebApp to change the settings.

:::info
Do not use both at once: changes are not synchronized between the two unless the WebApp and the TUI are manually refreshed.
:::

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

**✔️ All changes are saved automatically.**

When the apps indicate that an option has been changed (by a **\***), it means that the option has not yet been applied to the system, but the change has been saved in the configuration file. There is currently a bug causing some options to be declared as modified, even though they aren't.

import AsciinemaPlayerManual from '@site/src/components/AsciinemaPlayerManual';

<details>
    <summary>TUI Demo</summary>
    <AsciinemaPlayerManual
        src="/casts/tui_demo.cast"
        theme="monokai"
        autoplay={true}
        speed={1.5}
        fit="width"
    />
</details>

### Installing vs Enabling Services

In traditional Linux systems, the term "installing" is used to install software or services on systems like Ubuntu or Arch Linux. In nixblitz, we use the term "enabling" instead of "installing" because it makes more sense in the context of NixOS.

## Installation

The system is now ready to install. The system will download approximately 100MB of additional data.

<img src="img/install/060_install.png" alt="nixblitz installer boot menu" width="800"/>

Start the system installation by typing `y` and pressing Enter. This process will take a few minutes, depending on your system. The system will be built with your configuration and then installed.

The installation is now complete and there are a few options available:

<img src="img/install/070_post_install.png" alt="nixblitz post install boot menu" width="800"/>

Congratulations, your system is now ready to use!

<details>
    <summary>Full Installation Demo</summary>
    <AsciinemaPlayerManual
        src="/casts/install_demo.cast"
        theme="monokai"
        autoplay={true}
        speed={1.5}
        fit="width"
    />
</details>
