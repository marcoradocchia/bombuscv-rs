#!/bin/bash

# TODO: detect if script is being run as root and error if so.

# Colored output.
RED="\e[1;31m"
YELLOW="\e[1;33m"
GREEN="\e[1;32m"
CYAN="\e[1;36m"
NORM="\e[0m"

# Swap file path on RaspberryPi OS.
SWAP_FILE=/etc/dphys-swapfile

do_print_welcome=true
do_update=true
do_ask_reboot=true

usage() 
{
  printf "Usage: $0 [OPTIONS]\n\n"
  printf "OPTIONS:\n"
  printf "  -h  Print help information\n"
  printf "  -m  Mute welcome message\n"
  printf "  -u  Prevent script from updating the system\n"
  printf "  -r  Prevent script from asking for reboot\n"
}

ask_reboot() 
{
  printf $GREEN"Please reboot your system before continung. Reboot now? [N/y]\n"
  printf $GREEN"==> "$NORM

  read selection # Read standard input.
  case $selection in
    Y|y)
      systemctl reboot
      ;;
    *)
      printf $YELLOW"==> Warning:$NORM changes will only be applied at next reboot\n"
      ;;
  esac
}

greet ()
{
  printf "$CYAN\n#######################################################\n"
  printf        "## Congratulations! BombusCV successfully installed! ##\n"
  printf        "#######################################################\n$NORM"
}

# Print error message and exit.
exit_msg()
{
  printf $RED"==> Error:$NORM $1.\n"
  exit 1
}

welcome_msg()
{
  printf "$YELLOW██████╗  ██████╗ ███╗   ███╗██████╗ ██╗   ██╗███████╗ ██████╗██╗   ██╗\n"
  printf        "██╔══██╗██╔═══██╗████╗ ████║██╔══██╗██║   ██║██╔════╝██╔════╝██║   ██║\n"
  printf   "$NORM██████╔╝██║   ██║██╔████╔██║██████╔╝██║   ██║███████╗██║     ██║   ██║\n"
  printf        "██╔══██╗██║   ██║██║╚██╔╝██║██╔══██╗██║   ██║╚════██║██║     ╚██╗ ██╔╝\n"
  printf "$YELLOW██████╔╝╚██████╔╝██║ ╚═╝ ██║██████╔╝╚██████╔╝███████║╚██████╗ ╚████╔╝ \n"
  printf        "╚═════╝  ╚═════╝ ╚═╝     ╚═╝╚═════╝  ╚═════╝ ╚══════╝ ╚═════╝  ╚═══╝  \n\n$NORM"

  printf "$CYAN#####################################################################\n"
  printf      "## Installation helper script for bombuscv-rs (by Marco Radocchia) ##\n"
  printf      "## Warning: the installation process may take a while (>1h)...     ##\n"
  printf      "#####################################################################\n\n$NORM"
}

[ $USER = root ] && exit_msg "please don't run the script as root, run as normal user"

# Get CLI options/arguments.
while getopts "m?h?u?r?" opt; do
  case $opt in
    m) # Mute welcome message.
      do_print_welcome=false
      ;;
    u) # Prevent script from updating the system.
      do_update=false
      ;;
    r) # Prevent script from asking for reboot.
      do_ask_reboot=false
      ;;
    h) # Print help information.
      usage && exit 0
      ;;
  esac
done

# Print welcome message unless -m option specified.
[ $do_print_welcome = true ] && welcome_msg

# Check if Raspberry Pi is running RaspberryPi OS 64 bits:
command -v apt-get > /dev/null
[ $(uname -m) != "aarch64" -o $? = 1 ] && \
  exit_msg "please install RaspberryPi OS 64 bits and retry"

# Check if Raspberry is at least 4GB RAM.
[ $(free --mebi | awk '/^Mem:/ {print $2}') -lt 3000 ] && \
  exit_msg "required at least 4GB of RAM"

# Update the system.
[ $do_update = true ] && {
  printf "$GREEN==> Updating the system...$NORM\n"
  sudo apt-get -y update && sudo apt-get -y upgrade
}

# Update bootloader.
printf "$GREEN==> Updating bootloader...$NORM\n"
sudo rpi-eeprom-update -a

# Bring gpu memory up to 256MB.
printf "$GREEN==> Increasing gpu memory...$NORM\n"
sudo sed -i /boot/config.txt -e s'/gpu_mem=.*/gpu_mem=256/'

# Increasing swap size.
printf "$GREEN==> Increasing swap size...$NORM\n"
# storing the original swap size for later restore
orig_swap=$(awk -F'=' '/CONF_SWAPSIZE=/ {print $2}' $SWAP_FILE)
sudo sed -i $SWAP_FILE -e s'/CONF_SWAPSIZE=.*/CONF_SWAPSIZE=4096/'
sudo /etc/init.d/dphys-swapfile restart

# Enable legacy camera support with raspi-config in non-interactive mode.
sudo raspi-config nonint do_legacy 0

# Install all dependencies with apt-get.
printf "$GREEN==> Installing dependencies...$NORM\n"
sudo apt-get install -y \
  clang \
  libclang-dev \
  build-essential \
  cmake \
  git \
  ffmpeg \
  unzip \
  pkg-config \
  libjpeg-dev  \
  libpng-dev \
  libavcodec-dev \
  libavformat-dev \
  libswscale-dev \
  libgstreamer1.0-dev \
  libgstreamer-plugins-base1.0-dev \
  gstreamer1.0-gl \
  libxvidcore-dev \
  libx264-dev \
  libtbb2 \
  libtbb-dev \
  libdc1394-22-dev \
  libv4l-dev \
  v4l-utils \
  libopenblas-dev \
  libatlas-base-dev \
  libblas-dev \
  liblapack-dev \
  gfortran \
  libhdf5-dev \
  libprotobuf-dev \
  libgoogle-glog-dev \
  libgflags-dev \
  protobuf-compiler

# Download OpenCV 4.6.0.
wget -O opencv.zip https://github.com/opencv/opencv/archive/4.6.0.zip
wget -O opencv_contrib.zip https://github.com/opencv/opencv_contrib/archive/4.6.0.zip
# unzip downloaded files
unzip opencv.zip
unzip opencv_contrib.zip
# rename directories for convenience
mv opencv-4.6.0 opencv
mv opencv_contrib-4.6.0 opencv_contrib
# remove the zip files
rm opencv.zip
rm opencv_contrib.zip
# create the build directory
cd opencv && mkdir build && cd build

# Compile OpenCV 4.6.0.
printf "$GREEN==> Compiling OpenCV v4.6.0...$NORM\n"
# run cmake
cmake -DCMAKE_BUILD_TYPE=RELEASE \
-DCMAKE_INSTALL_PREFIX=/usr/local \
-DOPENCV_EXTRA_MODULES_PATH=$HOME/opencv_contrib/modules \
-DCPU_BASELINE=NEON \
-DENABLE_NEON=ON \
-DWITH_OPENMP=ON \
-DWITH_OPENCL=OFF \
-DBUILD_TIFF=ON \
-DWITH_FFMPEG=ON \
-DWITH_TBB=ON \
-DBUILD_TBB=ON \
-DWITH_GSTREAMER=ON \
-DBUILD_TESTS=OFF \
-DWITH_EIGEN=OFF \
-DWITH_V4L=ON \
-DWITH_LIBV4L=ON \
-DWITH_VTK=OFF \
-DWITH_QT=OFF \
-DWITH_GTK=OFF \
-DHIGHGUI_ENABLE_PLUGINS=OFF \
-DWITH_WIN32UI=OFF \
-DWITH_DSHOW=OFF \
-DWITH_AVFOUNDATION=OFF \
-DWITH_MSMF=OFF \
-DWITH_TESTs=OFF \
-DOPENCV_ENABLE_NONFREE=ON \
-DINSTALL_C_EXAMPLES=OFF \
-DINSTALL_PYTHON_EXAMPLES=OFF \
-DINSTALL_ANDROID_EXAMPLES=OFF \
-DWITH_ANDROID_MEDIANDK=OFF \
-DINSTALL_BIN_EXAMPLES=OFF \
-DOPENCV_GENERATE_PKGCONFIG=ON \
-DBUILD_EXAMPLES=OFF \
-DBUILD_JAVA=OFF \
-DBUILD_FAT_JAVA_LIB=OFF \
-DBUILD_JAVA=OFF \
-DBUILD_opencv_python2=OFF \
-DBUILD_opencv_python3=OFF \
-DENABLE_PYLINT=OFF \
-DENABLE_FLAKE8=OFF \
..
# Run make (compile) using all 4 cores.
make -j4

# Install OpenCV 4.6.0
printf "$GREEN==> Installing OpenCV v4.6.0...$NORM\n"
sudo make install
sudo ldconfig

# changing cwd back to $HOME 
cd $HOME

# Remove opencv source directories.
printf "$GREEN==> Removing OpenCV files...$NORM\n"
rm -rf $HOME/opencv
rm -rf $HOME/opencv_contrib

# Install rustup if cargo isn't on system.
command -v cargo > /dev/null || {
  printf "$GREEN==> Installing rustup...$NORM\n"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
}

# Cargo install bombuscv-rs if rustup successfully installed cargo.
command -v cargo > /dev/null
if [ $? = 0 ]; then 
  printf "$GREEN==> Installing bombuscv-rs...$NORM\n"
  $HOME/.cargo/bin/cargo install bombuscv-rs
else
  exit_msg "unable to install rustup, please retry"
fi

# Restoring swap size
printf "$GREEN==> Restoring swap size...$NORM"
sudo sed -i $SWAP_FILE -e s"/CONF_SWAPSIZE=.*/CONF_SWAPSIZE=$orig_swap/"
sudo /etc/init.d/dphys-swapfile restart

# Check if binary is installed successfully & greet if so.
command -v bombuscv > /dev/null && greet

# Ask for reboot unless -r option specified.
[ $do_ask_reboot = true ] && ask_reboot
