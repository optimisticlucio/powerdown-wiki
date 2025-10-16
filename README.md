# powerdown-wiki
The backend for the powerdown.wiki website. This documentation is gonna be... *not great*, but I'm gonna try writing out anything I feel is essential.

## Setup

To run this whole shebang you're gonna need to install docker, and then run `docker compose up`. You won't need anything else, because the docker containers will handle any dependencies. I recommend also writing an enviroment variables file (`.env`) at the root with all the variables listed in the [Enviroment Variables](#enviroment-variables) section. The .gitignore file lists ".env", so it shouldn't sync. *This is a good thing.* If you upload an .env file to the repo I swear to *god* I will find you.

If you wanna use the import tool, you'll need rust installed on your machine. Go to `/import-tool/`, and run the command `cargo run`.

## Enviroment Variables

There are some enviroment variables that are required when the docker compose call is made, and others that are optional.

### Required Variables

If these are missing when you run docker compose is made, you *will* get a crash at some point of normal operation. Not maybe, will.

- POSTGRES_USER, POSTGRES_PASSWORD: The user and password used when creating the postgres DB. Given to both the rust app and the postgres container.

- AWS_URL, AWS_ACCESS_KEY, AWS_SECRET_ACCESS_KEY: Information regarding the location and login data for AWS services, so we can use S3 buckets to store images and video and somesuch. *During development, point this at a LocalStack instance.*

- DISCORD_OAUTH2_CLIENT_ID, DISCORD_OAUTH2_CLIENT_SECRET: Discord client authentication data for OAuth2. You can get those in the [Discord Developer Portal](https://discord.com/developers/applications). We're using OAuth2 because I do not want to deal with passwords.

### Optional Variables

These variables aren't *required*, as they have default values incase they're missing. Still, you might need them for some case or another.

Each variable will have its default value listed in parantheses.

#### Bucket Names

The various bucket names in S3 for storing the PD-related data. Because every bucket name is unique, you will probably need to change some of these.

- S3_ART_BUCKET_NAME (powerdown-art-storage): The bucket where all the art section data is stored, like thumbnails and images. Should be public access.

- S3_CHARACTER_BUCKET_NAME (powerdown-character-storage): The bucket where the character-related art is stored. Should be public access.

- S3_MISC_BUCKET_NAME (powerdown-miscellaneous-storange): The bucket where anything that doesn't fit either of the other two categories goes, like the actual misc category or images that go in stories and such. Should be public access.

- S3_SQL_BACKUP_BUCKET_NAME (powerdown-sql-backups-storage): The bucket with backups of the various SQL tables we have going, and it'll read from there to see if there's an existing backup to read from on startup. **Shouldn't be public access**, unless you fancy random people being able to access your OAuth2 secret keys.
