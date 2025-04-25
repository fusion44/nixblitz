# Check for OVMF Code Path
let OVMF_CODE_PATH = if ($env | get -i OVMF_CODE_PATH | is-not-empty) {
    # 1. Use environment variable if it exists and is not empty
    let path_from_env = $env | get -i OVMF_CODE_PATH
    $path_from_env
} else if ('/run/libvirt/nix-ovmf/OVMF_CODE.fd' | path exists) {
    # 2. Fallback to path provided by libvirtd module (if enabled and running)
    let path_from_run = '/run/libvirt/nix-ovmf/OVMF_CODE.fd'
    $path_from_run
} else {
    # Error if none are found
    error make { msg: "OVMF Code Path not found. Set $env.OVMF_CODE_PATH, or ensure libvirtd is configured with OVMF, or ensure /etc/qemu/ovmf_code.path exists." }
}

# Check for OVMF Vars Template Path
let OVMF_VARS_TEMPLATE_PATH = if ($env | get -i OVMF_VARS_TEMPLATE_PATH | is-not-empty) {
    let path_from_env = $env | get -i OVMF_VARS_TEMPLATE_PATH
    $path_from_env
} else if ('/run/libvirt/nix-ovmf/OVMF_VARS.fd' | path exists) {
    let path_from_run = '/run/libvirt/nix-ovmf/OVMF_VARS.fd'
    $path_from_run
} else {
    error make { msg: "OVMF Vars Template Path not found. Set $env.OVMF_VARS_TEMPLATE_PATH, or ensure libvirtd is configured with OVMF, or ensure /etc/qemu/ovmf_vars.path exists." }
}

# Create a unique path for the writable copy for this specific VM instance
let VM_OVMF_VARS_PATH = $"./my-vm-ovmf-vars-($env.USER? | default 'user')-($env.PWD | hash md5 | str substring 0..7).fd"

# Copy the template to the writable location, handling potential errors
try {
    cp $OVMF_VARS_TEMPLATE_PATH $VM_OVMF_VARS_PATH
    # Make the copy writable for the owner (u+w = user add write)
    chmod u+w $VM_OVMF_VARS_PATH
    print $"Created writable OVMF vars file: ($VM_OVMF_VARS_PATH)"
} catch { |err|
    error make { msg: $"Failed to copy OVMF Vars template from '($OVMF_VARS_TEMPLATE_PATH)' to '($VM_OVMF_VARS_PATH)': ($err)" }
}

# --- Run QEMU Command ---
print "Starting QEMU VM..."
(qemu-system-x86_64 -enable-kvm -m 16384 -smp 4
  -serial file:/tmp/serial.log
  -debugcon file:/tmp/debugcon.log
  # --- UEFI Firmware ---
  # Use the determined paths
  -drive $"if=pflash,format=raw,readonly=on,file=($OVMF_CODE_PATH)"
  -drive $"if=pflash,format=raw,file=($VM_OVMF_VARS_PATH)"
  # --- Networking ---
  -netdev user,id=mynet0,hostfwd=tcp::10022-:22
  -device virtio-net-pci,netdev=mynet0
  # --- Drive ---
  -drive file=nixblitz-disk.qcow2,if=none,id=virtio0,format=qcow2
  -device virtio-blk-pci,drive=virtio0,bootindex=1
)

# --- Cleanup (Optional but Recommended) ---
# Remove the copied vars file after the QEMU process finishes
# Use '-f' to ignore errors if the file somehow doesn't exist
rm -f $VM_OVMF_VARS_PATH
print $"Cleaned up writable OVMF vars file: ($VM_OVMF_VARS_PATH)"
