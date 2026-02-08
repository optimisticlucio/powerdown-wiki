# powerdown-wiki

The backend for the powerdown.wiki website. This documentation is gonna be... *not great*, but I'm gonna try writing out anything I feel is essential.

## Setup

To run this whole shebang you're gonna need to install docker, and then run `docker compose up`. You won't need anything else, because the docker containers will handle any dependencies. I recommend also writing an enviroment variables file (`.env`) at the root with all the variables listed in the [Enviroment Variables](#enviroment-variables) section. The .gitignore file lists ".env", so it shouldn't sync. *This is a good thing.* If you upload an .env file to the repo I swear to *god* I will find you.

If you wanna use the import tool, you'll need rust installed on your machine. Go to `/import-tool/`, and run the command `cargo run`.

## Enviroment Variables

There are some enviroment variables that are required when the docker compose call is made, and others that are optional.

This entire section assumes you are using the docker_compose.yml provided at the repository root. If you're not, you might need to tinker with some of these values.

### Required Variables

If these are missing when you run docker compose is made, you *will* get a crash at some point of normal operation. Not maybe, will.

- `WEBSITE_URL`: The URL of the website people are visiting. Needed for stuff like OAuth2 redirect URLs. Do not include trailing slash.

- `POSTGRES_USER`, `POSTGRES_PASSWORD`: The user and password used when creating the postgres DB. Given to both the rust app and the postgres container.

- `AWS_ENDPOINT_URL`, `AWS_ACCESS_KEY`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`: Information regarding the location and login data for AWS services, so we can use S3 buckets to store images and video and somesuch. *During development, point this at a LocalStack instance.*

- `DISCORD_OAUTH2_CLIENT_ID`, `DISCORD_OAUTH2_CLIENT_SECRET`: Discord client authentication data for OAuth2. You can get those in the [Discord Developer Portal](https://discord.com/developers/applications). We're using OAuth2 because I do not want to deal with passwords.

- `GOOGLE_OAUTH2_CLIENT_ID`, `GOOGLE_OAUTH2_CLIENT_SECRET`: Ditto, for google instead. Get them in the [Google Auth Platform Dashboard](https://console.cloud.google.com/auth/clients).

- `GITHUB_OAUTH2_CLIENT_ID`, `GITHUB_OAUTH2_CLIENT_SECRET`: You get the idea. [Get them here](https://github.com/settings/developers).

### Optional Variables

These variables aren't *required*, as they have default values incase they're missing. Still, you might need them for some case or another.

Each variable will have its default value listed in parantheses.

#### To Modify on Production

- `BACKEND_IMAGE`: If this is set to anything, the backend rust image is pulled from the listed docker image, rather than compiling it locally. If not set, it just runs rust chef and compiles it on the given machine. **You probably want to set this to `ghcr.io/optimisticlucio/powerdown-wiki:latest` in production.** I set it up like this rather than making two different docker files so if I change something in the docker-compose regarding ENV or whatever I won't need to go back and update it on a second docker compose. Keeping consistency between production and testing enviroments yknow?

#### General

- `DEBUG`: If this is set to *anything*, it triggers debug mode. Currently, it only adds a giant banner on the site saying "THIS IS A TEST SITE". Obviously this should be unset in production.
- `COMPOSE_PROFILES`: Not one I made, it's from docker; if set to `development`, it'll start up the localstack image for local development. If unset or set to anything else, you only get the rust and postgres images.
- `DISABLE_MIGRATIONS`: If set to anything, SQL migrations are not run when the server starts up. By default, migrations are checked on every startup. Frankly I'm not sure why you'd wanna do this, but ais suggested to have this.
- `WEB_PORT`: The port that should be accessed to see the website. By default it's set to port 8080.
- `S3_PUBLIC_FACING_URL`: Incase `AWS_ENDPOINT_URL` isn't accessible to the end user (for example, if you're using localstack). If unset, the server will attempt to build an S3 URL from the `AWS_REGION` and bucketname on request.

#### Bucket Names

The various bucket names in S3 for storing the PD-related data. Because every bucket name is unique, you will probably need to change some of these.

- `S3_PUBLIC_BUCKET_NAME` (`powerdown-public-storage`): The bucket where we store everything the average user may run into during regular browsing - art, character thumbnails, videos, user pfps, that sort of deal. As the name indicates, should be public access.

- `S3_SQL_BACKUP_BUCKET_NAME` (`powerdown-sql-backups-storage`): The bucket with backups of the various SQL tables we have going, and it'll read from there to see if there's an existing backup to read from on startup. **Shouldn't be public access**, unless you fancy random people being able to access your OAuth2 access keys.

- `S3_PUBLIC_BUCKET_URL` (Not set by default): If this is set, gives users this URL to view items from the public bucket, rather than pointing to `AWS_ENDPOINT_URL`. For example, by default a url to see `ITEM_KEY` would be `AWS_ENDPOINT_URL`/`S3_PUBLIC_BUCKET_NAME`/`ITEM_KEY`, but if this is set, it would be `S3_PUBLIC_BUCKET_URL`/`ITEM_KEY`

## Cookies

There are some cookies that we use across the site. Here's the full list of them.

You should assume, whenever possible, that these keys aren't assigned. They're cookies, it's very hard to enforce their existence.

- `USER_SESSION_ID`: The session ID of the user currently logged in; self-explanatory. This cookie should be set to expire before the server's expiration date for the same session hits, purely for convenience.

- `NSFW_WARNING_SHOWN`: This cookie is set if the user has been shown the NSFW "are you sure you want to see this" warning and said "yes, I do." It should have an expiry date of about a day. If the cookie is set at all, we can assume its value is true.

## Arbitrary Values

In the DB there's a table called `arbitrary_value`, which holds key-value pairs. Both the key and value are strings, both cannot be null. Here's the list of the keys we have, and the expected values.

- `discord_invite_url`: A link which invites the user to the discord server. If the value is an empty string, the server is assumed to be on lockdown (no one can join from a public link).
