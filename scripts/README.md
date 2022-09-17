# VM Setup on Linux

Install [Quickemu](https://github.com/quickemu-project/quickemu)

## Windows Setup

1. Follow Windows setup instructions on the Quickemu docs
2. Install [Rust](https://www.rust-lang.org/tools/install)
3. Install C++ build tools using the [Visual Studio Installer](https://visualstudio.microsoft.com/downloads/)
4. Install [git for Windows](https://gitforwindows.org/)
5. Enable the OpenSSH server
   1. Open the Settings app.
   2. Click on Apps then click on Optional features (should be a small blue link near the middle of the screen).
   3. Find OpenSSH Server and install it.
   4. Reboot
   5. Open the Services app and find a service called OpenSSH SSH Server.
   6. Right click it and start the service.
   7. Optionally, click on properties and change the startup type to automatic so it starts on boot.
   8. Quickemu should've already configured the settings so SSH will work without additional configuration.
   9. Look for the windows-10.ports file in the windows-10 folder created by Quickemu on the host machine. It should have an entry like this: `ssh,22220` that specifies the ssh port.
   10. Try to ssh into the VM from the host machine like this: `ssh -p 22220 Quickemu@localhost`. When prompted for the password, it should be `quickemu`. (Quickemu docs explain this bit as well)
6. Install rsync
   1. Open the [msys download page](https://repo.msys2.org/msys/x86_64/).
   2. Download rsync-{CURRENT_VERSION}.pkg.tar.zst.
   3. Download libxxhash-{CURRENT_VERSION}.pkg.tar.zst.
   4. Download libzstd-{CURRENT_VERSION}.pkg.tar.zst.
   5. Download and install [PeaZip](https://peazip.github.io/peazip-64bit.html) to extract the files.
   6. Extract each archive using PeaZip. You will need to extract twice, once to extract the tarball from the zst archive, and once more to extract the files from the tarball.
   7. Each package should have a `usr` folder. Copy the contents from each `usr` folder into the `usr` folder used by Git (should be C:\Program Files\Git\usr).
   8. Add C:\Program Files\Git\usr\bin to your PATH environment variable using the Windows environment variable settings app.
   9. Open a terminal and verify rsync works.

Everything should be working at this point. Use the `rsync-from-windows` script to copy from the Windows VM to the host machine and use `rsync-to-windows` to do the opposite.

## Mac Setup

1. Follow Mac setup instructions on the Quickemu docs
2. Install [Rust](https://www.rust-lang.org/tools/install)
3. Enable the OpenSSH server
   1. Open System Preferences -> Sharing and then check Remote Login. This should enable ssh access.
   2. Reboot.
   3. When running the Quickemu script, make sure to change the port to 22221 using the `--ssh-port` flash so it doesn't conflict with the Windows port.
