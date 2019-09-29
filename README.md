# LLHD Website

This is the home of the [LLHD website](https://llhd.io/).

## Local Server

To run the server, do the following:

    cd server
    cargo run

### macOS Docker Fix

On macOS create a separate temporary directory such that docker can mount it without trouble:

    cd server
    mkdir tmp
    TMPDIR=$PWD/tmp cargo run

## Frontend

The frontend lives in the `frontend/` directory and is just a static HTML, CSS, and JS file.

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

```
docker build -t llhd-sandbox backend
```

## Deployment on Ubuntu

Run the following as root to set up an Ubuntu server instance (e.g. Amazon Lightsail/EC2).

### Dependencies
```
apt-get update
apt-get upgrade -y
apt-get install git awscli nginx

apt-get install docker-ce

service docker restart
usermod -a -G docker ubuntu

# Set aside disk space
fallocate -l 1G /swap.fs
chmod 0600 /swap.fs
mkswap /swap.fs

# Configure mount points
cat >>/etc/fstab <<EOF
/swap.fs        none            swap   sw       0   0
/playground.fs /mnt/playground  ext3   loop     0   0
EOF

# Reboot instance at this point.
```
