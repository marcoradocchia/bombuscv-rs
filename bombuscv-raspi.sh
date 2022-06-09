echo "██████╗  ██████╗ ███╗   ███╗██████╗ ██╗   ██╗███████╗ ██████╗██╗   ██╗"
echo "██╔══██╗██╔═══██╗████╗ ████║██╔══██╗██║   ██║██╔════╝██╔════╝██║   ██║"
echo "██████╔╝██║   ██║██╔████╔██║██████╔╝██║   ██║███████╗██║     ██║   ██║"
echo "██╔══██╗██║   ██║██║╚██╔╝██║██╔══██╗██║   ██║╚════██║██║     ╚██╗ ██╔╝"
echo "██████╔╝╚██████╔╝██║ ╚═╝ ██║██████╔╝╚██████╔╝███████║╚██████╗ ╚████╔╝ "
echo "╚═════╝  ╚═════╝ ╚═╝     ╚═╝╚═════╝  ╚═════╝ ╚══════╝ ╚═════╝  ╚═══╝  "

SWAP_FILE=/etc/dphys-swapfile

echo "#####################################################################"
echo "## Installation helper script for bombuscv-rs (by Marco Radocchia) ##"
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

# TODO: do I need more memory?
# TODO: If so enlarge memory using swap.

# Download & Compile OpenCV 4.5.5.
echo "==> Compiling OpenCV v4.5.5..."
cd $HOME
wget -O opencv.zip https://github.com/opencv/opencv/archive/4.5.5.zip
wget -O opencv_contrib.zip https://github.com/opencv/opencv_contrib/archive/4.5.5.zip
# unzip downloaded files
unzip opencv.zip
unzip opencv_contrib.zip
# rename directories for convenience
mv opencv-4.5.5 opencv
mv opencv_contrib-4.5.5 opencv
# remove the zip files
rm opencv.zip
rm opencv_contrib.zip
# create the build directory
cd opencv
mkdir build
cd build
# run cmake
cmake -D CMAKE_BUILD_TYPE=RELEASE \
  -D CMAKE_INSTALL_PREFIX=/usr/local \
  -D OPENCV_EXTRA_MODULES_PATH=$HOME/opencv_contrib/modules \
  -D CPU_BASELINE=NEON \
  -D ENABLE_NEON=ON \
  -D WITH_OPENMP=ON \
  -D WITH_OPENCL=OFF \
  -D BUILD_TIFF=ON \
  -D WITH_FFMPEG=ON \
  -D WITH_TBB=ON \
  -D BUILD_TBB=ON \
  -D WITH_GSTREAMER=ON \
  -D BUILD_TESTS=OFF \
  -D WITH_EIGEN=OFF \
  -D WITH_V4L=ON \
  -D WITH_LIBV4L=ON \
  -D WITH_VTK=OFF \
  -D WITH_QT=OFF \
  -D WITH_GTK=OFF \
  -D HIGHGUI_ENABLE_PLUGINS \
  -D WITH_WIN32UI=OFF \
  -D WITH_DSHOW=OFF \
  -D WITH_AVFOUNDATION=OFF \
  -D WITH_MSMF=OFF \
  -D WITH_TESTs=OFF \
  -D OPENCV_ENABLE_NONFREE=ON \
  -D INSTALL_C_EXAMPLES=OFF \
  -D INSTALL_PYTHON_EXAMPLES=OFF \
  -D INSTALL_ANDROID_EXAMPLES=OFF \
  -D WITH_ANDROID_MEDIANDK=OFF \
  -D INSTALL_BIN_EXAMPLES=OFF \
  -D OPENCV_GENERATE_PKGCONFIG=ON \
  -D BUILD_EXAMPLES=OFF \
  -D BUILD_JAVA=OFF \
  -D BUILD_FAT_JAVA_LIB=OFF \
  -D BUILD_JAVA=OFF \
  -D BUILD_opencv_python2=OFF \
  -D BUILD_opencv_python3=OFF \
  -D ENABLE_PYLINT=OFF \
  -D ENABLE_FLAKE8=OFF \
  ..
# run make using all 4 cores
make -j4

# Install OpenCV 4.5.5
echo "==> Installing OpenCV v4.5.5..."
sudo make install
sudo ldconfig
# NOTE: most likely unecessary
sudo apt-get -y update && sudo apt-get -y upgrade

# Remove opencv source directories.
echo "==> Removing OpenCV files..."
rm -rf $HOME/opencv
rm -rf $HOME/opencv_contrib

# Restoring swap size
echo "==> Restoring swap size..."
sudo sed -i $SWAP_FILE -e s"/CONF_SWAPSIZE=.*/CONF_SWAPSIZE=$orig_swap/"
sudo /etc/init.d/dphys-swapfile restart

# Install rustup.
echo "==> Installing rustup..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Source the shell configuration to update the PATH.
# BUG: what if the script was not launched with bash or the bashrc is not the
# one indicated?
source $HOME/.bashrc

# Cargo install bombuscv-rs if rustup successfully installed cargo.
echo "==> Installing bombuscv-rs..."
[ $(command -v cargo | wc -l) ] cargo install bombuscv-rs
