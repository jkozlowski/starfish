# How to run SPDK apps

* Goal: get rust code that runs some simple spdk code to compile and run.

## My laptop

* Vagrant box from http://www.spdk.io/doc/vagrant.html doesn't want to run on my MacOSX
* Somehow this needs to tie in https://www.qemu.org/ with some virtual machines;
  qemu is used to emulate a NVME drive.
* There is also this: https://hub.docker.com/r/ljishen/spdk/; somehow I think this packages
  the vagrant box from spdk into a docker container; but it also does not want to run.
* http://lightnvm.io/
* https://openchannelssd.readthedocs.io/en/latest/qemu/ <- this mentions qemu

So:

#### Run VirtualBox and inside it install KVM and run standard Vagrant box from SPDK <- Not going to work :(

* Not going to work, since VirtualBox does not pass VT-X/AMD-V to the guest: https://askubuntu.com/questions/328748/how-to-enable-nested-virtualization-in-ubuntu

#### Run QEMU on mac with a VM and then install KVM inside that VM

```
$ brew install qemu libvirt
$ vagrant plugin install vagrant-libvirt
```

* https://www.emaculation.com/doku.php/ppc-osx-on-qemu-for-osx
* https://github.com/rancher/vm
* https://github.com/vagrant-libvirt/vagrant-libvirt/issues/497
  \*\* https://github.com/vagrant-libvirt/vagrant-libvirt/issues/497#issuecomment-331226071

#### Try to run Vagrant on QEMU/libvirt and see if KVM is enabled

* https://github.com/vagrant-libvirt/vagrant-libvirt
* https://gist.github.com/rhuss/182bc90dd8b2c5ace6db
* https://github.com/vagrant-libvirt/vagrant-libvirt/issues/445

TODO:

* Try running in AWS <- this will let me know if I'm doing something wrong.
* Once that is working, figure out how to stick the same env in VirtualBox.

## AWS

* https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/nvme-ebs-volumes.html
* Looks like I can afford a lot of days of compute on C5 instances; maybe I should do that
* The thing to figure out is how I can dev locally still, because I don't want to run VNC, unless I absolutely have to.
