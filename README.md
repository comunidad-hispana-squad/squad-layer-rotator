# Squad Layer rotator

Reads from `LOCAL_FOLDER` a different file per day and publishes it into `SFTP_REMOTE_PATH`.

Uses SFPT as the transport protocol.

## Running

### Locally

The following packages are required to run the project
- pkgconf
- libssl-dev

Create an `.env` file
```
SFTP_HOST=192.168.1.51:2022
SFTP_USER=user
SFTP_PASSWORD=pa\$\$word
SFTP_REMOTE_PATH=/Remote/Folder/File
RUN_HOUR=0
```

```
cargo run
```

### Docker

```
docker run -it --rm -e "SFTP_HOST=..." ghcr.io/comunidad-hispana-squad/squad-laye
r-rotator:<version>
```


## Environmentals
- `SFTP_HOST`: Host & Port, example `192.168.1.51:22`.
- `SFTP_USER`: User for authentication.
- `SFTP_PASSWORD`: Password for authentication.
- `SFTP_REMOTE_PATH`: Path of the destinations file.
- `RUN_HOUR`: Hour to run the script. Goes from 0 to 23. Defaults to `99` which will cause to run on startup.
- `LOCAL_FOLDER`: Folder to retrieve files from. Defaults to `./layers`.

## File selection

File selection is done in a deterministic way: `dy % nf`

Where `dy` is days of year from 1 to 365 and `nf` is the number of files in the `LOCAL_FOLDER`

> Files to choose from 3 chosen index 1. Day 64
