Error starting domain: Hook script execution failed: internal error: Child process (LC_ALL=C PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/snap/bin /etc/libvirt/hooks/qemu Tiamat prepare begin -) unexpected exit status 1

Traceback (most recent call last):
  File "/usr/share/virt-manager/virtManager/asyncjob.py", line 72, in cb_wrapper
    callback(asyncjob, *args, **kwargs)
  File "/usr/share/virt-manager/virtManager/asyncjob.py", line 108, in tmpcb
    callback(*args, **kwargs)
  File "/usr/share/virt-manager/virtManager/object/libvirtobject.py", line 57, in newfn
    ret = fn(self, *args, **kwargs)
  File "/usr/share/virt-manager/virtManager/object/domain.py", line 1384, in startup
    self._backend.create()
  File "/usr/lib/python3/dist-packages/libvirt.py", line 1353, in create
    raise libvirtError('virDomainCreate() failed')
libvirt.libvirtError: Hook script execution failed: internal error: Child process (LC_ALL=C PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/snap/bin /etc/libvirt/hooks/qemu Tiamat prepare begin -) unexpected exit status 1
