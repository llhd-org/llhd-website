# LLHD Website

This is the home of the [LLHD website](http://llhd.io/).

## Local Server

To run the server, do the following:

    cd server
    cargo run

### macOS Docker Fix

On macOS create a separate temporary directory such that docker can mount it without trouble:

    cd server
    mkdir tmp
    env TMPDIR=$PWD/tmp cargo run

## Frontend

The frontend lives in the `frontend/` directory and is just a static HTML, CSS, and JS file.

To render the specification, run

    npm install
    npm run render

## Server

The web server lives in the `server/` directory. During development, you can place the following environment variables in a `.env` file on disk in that directory. In production, these should be set according to the deployment environment.

| Variable                | Required | Default        | Description                       |
|-------------------------|----------|----------------|-----------------------------------|
| `LLHD_WEBSITE_ROOT`     | Yes      |                | The path to the HTML/CSS/JS files |
| `LLHD_WEBSITE_ADDRESS`  | No       | 127.0.0.1      | The address to listen on          |
| `LLHD_WEBSITE_PORT`     | No       | 5000           | The port to listen on             |
| `LLHD_WEBSITE_LOG_FILE` | No       | access-log.csv | The file to record accesses to    |

## Backend

When the user types some HDL code into the interactive editor on the website, the moore compiler is run in the docker container defined in `backend/` to produce the corresponding LLHD output. The server looks for a `llhd-sandbox` container. Build as follows:

    docker build -t llhd-sandbox backend

## Deployment on Ubuntu

Run the following as root to set up an Ubuntu server instance (e.g. Amazon Lightsail/EC2).

### As root
```
apt-get update
apt-get upgrade -y
apt-get install build-essential curl git awscli nginx

# Allocate swap space
fallocate -l 1G /swap.fs
chmod 0600 /swap.fs
mkswap /swap.fs

# Set aside disk space
fallocate -l 512M /playground.fs
device=$(losetup -f --show /playground.fs)
mkfs -t ext3 -m 1 -v $device
mkdir /mnt/playground

# Configure mount points
cat >>/etc/fstab <<EOF
/swap.fs        none            swap   sw       0   0
/playground.fs /mnt/playground  ext3   loop     0   0
EOF

# Install docker
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu bionic stable"
apt update
apt-get install docker-ce
service docker restart
usermod -a -G docker ubuntu

# Install Rust
curl -sSf https://sh.rustup.rs | sh -s -- -y

# Reboot instance at this point.
reboot
```

### As regular user
```
# Clone the repository
cd /home/ubuntu
git clone https://github.com/llhd-org/www.llhd.io.git

# Build the server
cd /home/ubuntu/www.llhd.io/server
cargo build --release

# Build the docker container
cd /home/ubuntu/www.llhd.io
docker build -t llhd-sandbox backend

```

### As root
```
# Install the systemd service
ln -s /home/ubuntu/www.llhd.io/{server/llhd-io.service,backend/llhd-io-backend.*} /etc/systemd/system/
service llhd-io start
systemctl enable llhd-io.service
```
