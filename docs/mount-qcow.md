sudo modprobe nbd max_part=16

sudo qemu-nbd -c /dev/nbd0 /var/lib/libvirt/images/DragonDogma2.qcow2

sudo fdisk -l /dev/nbd0

sudo mount /dev/nbd0p3 /mnt
