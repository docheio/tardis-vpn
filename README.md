# Tardis VPN
Tardis is a high-speed VPN (not encrypted) for server-to-server communication.

## Usage
Install Tardis (Ubuntu)
```bash
sudo apt install -y cargo make
git clone https://github.com/docheio/tardis.git
cd tardis
make all
make install
```
Update Tardis
```bash
make update
make re
make install
```
Start service
```bash
# Must edit config „/etc/systemd/system/tardisd.service“
systemctl enable --now tardisd
```