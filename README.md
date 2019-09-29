# LLHD Website

This is the home of the [LLHD website](https://llhd.io/).

## Server

The web server lives in the `server/` directory. During development, you can place the following environment variables in a `.env` file on disk in that directory. In production, these should be set according to the deployment environment.

| Variable                | Required | Default        | Description                       |
|-------------------------|----------|----------------|-----------------------------------|
| `LLHD_WEBSITE_ROOT`     | Yes      |                | The path to the HTML/CSS/JS files |
| `LLHD_WEBSITE_ADDRESS`  | No       | 127.0.0.1      | The address to listen on          |
| `LLHD_WEBSITE_PORT`     | No       | 5000           | The port to listen on             |
| `LLHD_WEBSITE_LOG_FILE` | No       | access-log.csv | The file to record accesses to    |
