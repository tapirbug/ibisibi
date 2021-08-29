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
./build.sh $CROSS_TRIPLE || exit 1

# Copy to Raspberry Pi via scp
echo "Build successful, deploying binary to $RPI_HOSTNAME..."

# Stop service for upgrade first (if running)
ssh $SSH_HOST 'sudo systemctl stop ibisibi'

# Update unit file and executable, restart service
scp ibisibi.service $SCP_DEST && \
scp target/$CROSS_TRIPLE/release/ibisibi $SCP_DEST && \
ssh $SSH_HOST 'rm ~/bin/ibisibi; ln -s ~/ibisibi/ibisibi ~/bin/ibisibi && sudo mv -f ~/ibisibi/ibisibi.service /etc/systemd/system && sudo systemctl enable ibisibi && sudo systemctl start ibisibi'
