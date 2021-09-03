if test -d ".cargo"; then
  # installing from source
  cargo build --release || exit 1
  BINARY="$(pwd)/target/release/ibisibi"
elif test -x ibisibi; then
  # installing from release
  BINARY="$(pwd)/ibisibi"
else
  echo -e "Run either from source code or from release dir, exiting."
  exit 1
fi
CONFIG="$(pwd)/examples/robo.yaml"
if test ! -f $CONFIG; then
  echo -e "Config not found"
  exit 1
fi

mkdir -p ~/bin && \
ln -sf $BINARY ~/bin/ibisibi || exit 1

if systemctl is-active --quiet ibisibi.service; then
  sudo systemctl stop ibisibi
fi

sudo echo "[Unit]" > /etc/systemd/system/ibisibi.service && \
sudo echo "Description=ibisibi" >> /etc/systemd/system/ibisibi.service && \
sudo echo "Requires=" >> /etc/systemd/system/ibisibi.service && \
sudo echo "After=" >> /etc/systemd/system/ibisibi.service && \
sudo echo "" >> /etc/systemd/system/ibisibi.service && \
sudo echo "[Install]" >> /etc/systemd/system/ibisibi.service && \
sudo echo "WantedBy=multi-user.target" >> /etc/systemd/system/ibisibi.service && \
sudo echo "" >> /etc/systemd/system/ibisibi.service && \
sudo echo "[Service]" >> /etc/systemd/system/ibisibi.service && \
sudo echo "User=$USER" >> /etc/systemd/system/ibisibi.service && \
sudo echo "Type=simple" >> /etc/systemd/system/ibisibi.service && \
sudo echo "ExecStart=$HOME/bin/ibisibi run $CONFIG" >> /etc/systemd/system/ibisibi.service && \
sudo echo "Restart=always" >> /etc/systemd/system/ibisibi.service && \
sudo echo "RestartSec=10" >> /etc/systemd/system/ibisibi.service && \
sudo systemctl enable ibisibi && \
sudo systemctl start ibisibi
