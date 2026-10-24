# Hosting randomtask.ch

randomtask.ch (the web app) and api.randomtask.ch (the server) run together on one Infomaniak VPS, with Docker Compose:

- **`server`** is the randomtask server, with its SQLite database in the `server-data` volume.
- **`web`** is Caddy. It serves the web app at randomtask.ch, redirects www.randomtask.ch there, forwards api.randomtask.ch to the server, and gets HTTPS certificates for all three from Let's Encrypt.

The [Images workflow](../.github/workflows/images.yml) builds both images and publishes them to the GitHub container registry: tagged `main` from every push to main, and `vX.Y.Z` and `latest` from every release tag. The server only pulls them, so it needs neither Rust nor Node. CI's `production` job starts the same images on every pull request and checks that they answer.

## Setting it up

1. **Make the images public, once.** Under the profile's *Packages*, open `randomtask-server` and `randomtask-web`, and in *Package settings* change the visibility to public. (Otherwise the VPS has to `docker login ghcr.io` with a token that can read packages.)

2. **Order the VPS.** In the Infomaniak Manager, order a VPS (VPS Lite is enough to start) with Ubuntu 24.04 or Debian 12, and add your SSH key. If the VPS has a firewall in the Manager, allow TCP 22, 80 and 443, and UDP 443.

3. **Point the domain at it.** In the DNS zone of randomtask.ch, add `A` records for `randomtask.ch`, `www.randomtask.ch` and `api.randomtask.ch` with the VPS's IPv4 address, and `AAAA` records with its IPv6 address if it has one. Caddy can only get certificates once these resolve.

4. **Install Docker and get the Compose file.** Over SSH on the VPS:

   ```sh
   curl -fsSL https://get.docker.com | sudo sh
   git clone https://github.com/davidswords/randomtask.git
   cd randomtask/deploy
   ```

5. **Start it.**

   ```sh
   sudo docker compose pull
   sudo docker compose up --detach --no-build
   ```

   This runs the latest release. To run main instead, put `RANDOMTASK_TAG=main` in a `.env` file next to `compose.yaml` first.

6. **Check it.** <https://randomtask.ch> opens the app, and creating a free account there registers it with api.randomtask.ch. `sudo docker compose logs` shows what both are doing.

## Updating

After a release, or a push to main when running main:

```sh
cd randomtask/deploy
git pull
sudo docker compose pull
sudo docker compose up --detach --no-build
```

## Backing up the database

The database holds the accounts: the email addresses, the key parameters, the bcrypt hashes of the server passwords, and the sessions. It never holds passwords, keys or tasks. To copy it out, stop the server for a moment so the copy is consistent:

```sh
sudo docker compose stop server
sudo docker run --rm -v deploy_server-data:/data -v "$PWD":/backup debian:bookworm-slim \
  tar czf /backup/server-data-$(date +%F).tar.gz -C /data .
sudo docker compose start server
```

Then copy the file somewhere off the VPS.
