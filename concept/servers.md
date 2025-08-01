# Server Infrastructure

This is the current format I think would be good for the server, before I start working on it.

```mermaid
graph LR

source["Users"]
main["Main Code"]

subgraph Art Databases
    artdb["Metadata"]
    artfiledb["Image Files"]
    arttagdb["Tags"]
end


source --- main
main --- artdb
artdb --> artfiledb
arttagdb --> artdb
```

## Art Databases

The art databases will consist of four databases:

- One holding most metadata for each art post. (Post DB)
- One holding the tags of each post. (Tags DB)
- One holding the img/video files of the posts. (File DB)
- One holding links *to* the File DB, connecting it to the relevant post from Post DB (Image DB)

The Post database will be an SQLite db run on Cloudflare's D1 infrastructure. It'll be a single table, with the following columns:

```sql
ID int NOT NULL PRIMARY KEY,
Title varchar(255) NOT NULL,
Thumbnail tinytext NOT NULL, -- References a File DB url
-- HOW DO I HANDLE ARTISTS? JSON OR FOREIGN KEY?
CreationDate date NOT NULL,
LastEditDate timestamp NOT NULL, -- SHOULD NOT BE EDITABLE TO USERS!!
Format enum(IMAGE, VIDEO) NOT NULL -- I think I should change this. This does not play well with everything else. Maybe just set the format based on the contents of the urls? Whether they're .png or .mov or anything?
-- TO HANDLE: IMG LINKS
-- TO HANDLE: TAGS

```

The Tags database will be another D1 database, this one exclusively holding the tags of each art piece.

The File database will be a R2 Cloudflare database. They allow up to 10GB for free per month which is very nice.

TODO!

### Considerations

- What's the max length we expect a title to be? It shouldn't be too long for useability. Right now it's 255 just for the sake of it.
- How can we make sure this db will handle emojis and special characters appropriately?
