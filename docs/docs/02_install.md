# System Installation

There are multiple ways to install a nixblitz system:

1. Using the nixblitz ISO image
2. Using a standard NixOS ISO
3. Remotely via SSH (not yet implemented)

## 1. Using the nixblitz ISO image

### Download the ISO image

Version v0.1.0: https://zipline.f44.fyi/u/nixos-minimal-NIXBLITZ-x86_64-linux_2.iso

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

### Run the installer

For installation on a bare metal system, you can use the USB thumb drive installer. For installation on a VM, you can just attach the ISO image to the VM.

<Tabs groupId="system-type">
  <TabItem value="bare_metal" label="USB" default>

    ### Flash the ISO image

    Make sure you have a USB stick with at least 16GB of storage. If you want to run nixblitz in a VM, skip this step.


    <Tabs groupId="operating-system">
      <TabItem value="linux" label="Linux" default>
        #### Using Etcher
        Download Etcher from https://www.balena.io/etcher/ and use it to flash the ISO image to your USB stick.

        #### Using dd
        Open a terminal and run the following command:

        ```bash
        sudo dd if=NIXBLITZ-x86_64-linux_v0.1.0.iso of=/dev/sdX bs=4M status=progress
        ```

        Replace `/dev/sdX` with the path to your USB stick. Insert the USB stick into your device and power it on.
      </TabItem>
      <TabItem value="macos" label="MacOS">
        TODO
      </TabItem>
      <TabItem value="windows" label="Windows">
        #### Using Etcher
        Download Etcher from https://www.balena.io/etcher/ and use it to flash the ISO image to your USB stick.

        Insert the USB stick into your device and power it on.
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

### Boot the nixblitz

When finished, you should see the nixblitz installer splash screen:

<img src="img/install/010_boot_menu.png" alt="nixblitz installer boot menu" width="600"/>
Either hit the `Enter` key to boot into the installer or let the counter count down.
The system will not boot into the actual installer. When finished, you should see the nixblitz splash screen:

<img src="img/install/020_select_install_method.png" alt="nixblitz installer install method" width="600"/>
Choose `On this machine (local install)` and hit Enter. Remote install is not yet implemented.

:::info
At this point you can also SSH into the device and run the installer from there:

```bash
# with default port
ssh -oUserKnownHostsFile=/dev/null -oStrictHostKeyChecking=no nixos@192.168.0.101

# with custom port (when running in a VM etc)
ssh -oUserKnownHostsFile=/dev/null -oStrictHostKeyChecking=no nixos@192.168.0.101 -p 2222
```

Replace the ip with the IP of your device.

The default password is `nixblitz`
:::

Next, the installer will try to detect your hardware and show you what platform was detected:
<img src="img/install/030_detect_platform.png" alt="nixblitz installer boot menu" width="600"/>
In this case, we're running in a virtual machine, so it detected that. Hit `Enter` to continue.

<details>
    <summary>Wrong platform detected?</summary>

    Should the installer fail to detect your hardware, you can manually select the platform by pressing 'N' and then selecting the platform you're running on.

    <img src="img/install/031_select_custom_platform.png" alt="nixblitz Installer Boot Menu" width="600"/>

</details>

Next, the installer will ask you to select the disk to install nixblitz to:

<img src="img/install/040_select_install_disk.png" alt="nixblitz installer boot menu" width="600"/>
Hit `Enter` to continue.

The installer will now show you the nixblitz CLI to change any settings:
<img src="img/install/050_cli.png" alt="nixblitz installer boot menu" width="800"/>
Navigating the CLI:

- The currently focused pane is highlighted in red.
- `left right` or `hl` => navigate between panes.
- `up down ` or `jk` => navigate app and option list entries.
- `enter` => change an option; booleand will just toggle, others will open a popup.
- `tab` => switch elements in popups.
- `q` or `ctrl-c` => quit the app.

**✔️ All changes are saved automatically.**

When the TUI indicates that an options is changed (indicated by a **\***), it means that the option is not yet applied to the system, but the change is persisted in the configuration file.

import AsciinemaPlayerManual from '@site/src/components/AsciinemaPlayerManual';

<details>
    <summary>Demo</summary>
    <AsciinemaPlayerManual
        src="/casts/tui_demo.cast"
        theme="monokai"
        autoplay={true}
        speed={1.5}
        fit="width"
    />
</details>

#### Installing vs Enabling Services

In traditional Linux systems you might be used to "installing" software or services on systems like Ubuntu or Arch Linux.
In nixblitz, we use the term "enabling" instead of "installing" as it makes more sense in the context of NixOS.

### Installation

The system is now ready installed. The system will download approximately 100MB of additional data.

<img src="img/install/060_install.png" alt="nixblitz installer boot menu" width="800"/>

Type `y` and hit `Enter` to start the system installation. This process will take a few minutes, depending on your system. The system will now be built and installed.

When the install is finished you have a few options:

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
