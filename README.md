# install_gtk
A simple program that will automate installing GTK4 on Windows

# Note: GTK4 must be installed this way if you want run any of the Game Hacking Dojo applications

Automatic approach:
 - Double clicking the application.

Manual approach:
 - Downloading and installing latest vc runtime from "https://aka.ms/vs/17/release/vc_redist.x64.exe", if you don't have it already.
 - Downloading and installing MSYS2 from "https://www.msys2.org/" or "https://github.com/msys2/msys2-installer/releases/"
 - Running the bash file found at "C:\msys64\usr\bin\bash.exe" by default
 - Executing ```pacman -Syu``` and then ```pacman -S mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-glade mingw-w64-ucrt-x86_64-toolchain mingw-w64-ucrt-x86_64-pkg-config```
 - Adding "C:\msys64\ucrt64\bin" to the PATH in the system environment variables
