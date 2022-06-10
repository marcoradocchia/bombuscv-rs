echo "██████╗  ██████╗ ███╗   ███╗██████╗ ██╗   ██╗███████╗ ██████╗██╗   ██╗"
echo "██╔══██╗██╔═══██╗████╗ ████║██╔══██╗██║   ██║██╔════╝██╔════╝██║   ██║"
echo "██████╔╝██║   ██║██╔████╔██║██████╔╝██║   ██║███████╗██║     ██║   ██║"
echo "██╔══██╗██║   ██║██║╚██╔╝██║██╔══██╗██║   ██║╚════██║██║     ╚██╗ ██╔╝"
echo "██████╔╝╚██████╔╝██║ ╚═╝ ██║██████╔╝╚██████╔╝███████║╚██████╗ ╚████╔╝ "
echo "╚═════╝  ╚═════╝ ╚═╝     ╚═╝╚═════╝  ╚═════╝ ╚══════╝ ╚═════╝  ╚═══╝  "

# TODO: add colors! Green to ==> lines & red to Error:

SWAP_FILE=/etc/dphys-swapfile

echo "#####################################################################"
echo "## Installation helper script for bombuscv-rs (by Marco Radocchia) ##"
echo "## Requirement: RaspberryPi 4 (4/8GB), RaspberryPi OS aarch64      ##"
echo "#####################################################################"
echo

# Check if Raspberry Pi is running RaspberryPi OS 64 bits:
[ $(uname -m) != "aarch64" -o $(command -v apt-get | wc -l) != 1 ] && \
  echo "Error: please install RaspberryPi OS 64 bits and retry." && exit 1

# Check if Raspberry is at least 4GB RAM.
[ $(free --mebi | grep -e "^Mem:" | awk '{print $2}') -lt 3000 ] && \
  echo "Error: required at least 4GB of RAM." && exit 1

# Update the system.
echo "==> Updating the system..."
sudo apt-get -y update && sudo apt-get -y upgrade

# Update bootloader.
echo "==> Updating bootloader..."
sudo rpi-eeprom-update -a

# Bring gpu memory up to 256MB.
echo "==> Increasing gpu memory..."
sudo sed -i /boot/config.txt -e s'/gpu_mem=.*/gpu_mem=256/'

# Increasing swap size.
echo "==> Increasing swap size..."
# storing the original swap size for later restore
orig_swap=$(awk -F'=' '/CONF_SWAPSIZE=/ {print $2}' $SWAP_FILE)
sudo sed -i $SWAP_FILE -e s'/CONF_SWAPSIZE=.*/CONF_SWAPSIZE=4096/'
sudo /etc/init.d/dphys-swapfile restart

# Install all dependencies with apt-get.
echo "==> Installing dependencies..."
sudo apt-get install -y \
  clangd \
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

# Download OpenCV 4.5.5.
curl -o opencv.zip https://github.com/opencv/opencv/archive/4.5.5.zip
curl -o opencv_contrib.zip https://github.com/opencv/opencv_contrib/archive/4.5.5.zip
# unzip downloaded files
unzip opencv.zip
unzip opencv_contrib.zip
# rename directories for convenience
mv opencv-4.5.5 opencv
mv opencv_contrib-4.5.5 opencv_contrib
# remove the zip files
rm opencv.zip
rm opencv_contrib.zip
# create the build directory
cd opencv
mkdir build
cd build

# Compile OpenCV 4.5.5.
echo "==> Compiling OpenCV v4.5.5..."
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
# run make using all 4 cores
make -j4

# Install OpenCV 4.5.5
echo "==> Installing OpenCV v4.5.5..."
sudo make install
sudo ldconfig

# Remove opencv source directories.
echo "==> Removing OpenCV files..."
rm -rf $HOME/opencv
rm -rf $HOME/opencv_contrib

# Install rustup.
echo "==> Installing rustup..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Source the shell configuration to update the PATH.
source $HOME/.cargo/env

# Cargo install bombuscv-rs if rustup successfully installed cargo.
echo "==> Installing bombuscv-rs..."
[ $(command -v cargo | wc -l) ] cargo install bombuscv-rs

# Restoring swap size
echo "==> Restoring swap size..."
sudo sed -i $SWAP_FILE -e s"/CONF_SWAPSIZE=.*/CONF_SWAPSIZE=$orig_swap/"
sudo /etc/init.d/dphys-swapfile restart

# TODO: ask for reboot.
