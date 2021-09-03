#!/bin/sh
# Builds and deploys ibsiibi to a remote system running arm-unknown-linux-gnueabihf
# e.g. Raspberry Pi 2

# RPi2-compatible ARM
RPI_USER="pi"
RPI_HOSTNAME="busowitz.local"
RPI_DIR="~/ibisibi"
SSH_HOST="$RPI_USER@$RPI_HOSTNAME"
SCP_DEST="$SSH_HOST:$RPI_DIR"
CROSS_TRIPLE="arm-unknown-linux-gnueabihf"

# cd to fernspielapparat
cd $(cd -P -- "$(dirname -- "$0")" && pwd -P) && \

# And build for CROSS_TRIPLE
RELEASE_DIR_NAME=$(./build.sh $CROSS_TRIPLE)
if test -z $RELEASE_DIR; then
  echo -e "Build failed, exiting..."
  exit 1
fi

# Copy to Raspberry Pi via scp
echo "Build successful, deploying release to $RPI_HOSTNAME..."

# Stop service for upgrade first (if running)
ssh $SSH_HOST 'sudo systemctl stop ibisibi'

# Update unit file and executable, restart service
scp -r release/$RELEASE_DIR_NAME $SCP_DEST && \
ssh $SSH_HOST 'cd $RELEASE_DIR_NAME && ./install.sh'
