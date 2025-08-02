# Server Infrastructure

This is the current format I think would be good for the server, before I start working on it.

---

## Code

TODO

---

## Databases

The databases will be mostly SQLite tables stored on Cloudflare's D1 service, and partially file hosting databases on the R2 service.

```mermaid
graph LR

creator["Creator"]


subgraph Art
    artdb["Post"]
    artimagedb["Image"]
    arttagdb["Tags"]
    arttoartistdb["Art to Artist"]

    artdb --> artimagedb
    arttagdb --> artdb
    arttoartistdb --> artdb
end

subgraph User
    user["User"]
    usersession["Session"]
    openid["OpenID"]
    
    usersession --> user
    openid --> user
end


arttoartistdb --> creator
creator --> user


```

---

### User

The user databases will consist of the following:

- **User DB,** holding the info regarding the users registered to the site.
- **OpenID DB,** holding the indentifiers for each user on various OpenID protocols I implemented for login. (Initially just github or discord, but making it a table already to not bite it later.)
- **Session DB,** holding the current sessions of each user.

The User DB will include the following columns:

```sql
    ID int NOT NULL PRIMARY KEY,
    Username varchar(30) NOT NULL,
```

The OpenID DB will include the following columns:

```sql
    TokenIssuer varchar(255? /*how long?*/) NOT NULL,
    IssuerUserID varchar(255?) NOT NULL,
    LocalUserID int NOT NULL FOREIGN KEY REFERENCES User(ID),
```

#### Considerations

- What else do we want users to be able to store about themselves?

### Art

The art databases will consist of the following:

- **Post DB,** holding the metadata that's 1:1 for each post. Title, thumbnail, etc.
- **Tag DB,** holding the tags of each post.
- **File DB,** holding the img/video files of the posts.
- **Image DB,** holding links *to* the File DB, connecting it to the relevant post from Post DB.
- **ArtToArtist DB,** holding foreign keys from Post DB and Creator DB, connecting art with artist. (This will allow searching art by artist, and having more than one artist per art piece.)

The Post database will include the following columns:

```sql
ID int NOT NULL PRIMARY KEY,
Title varchar(255) NOT NULL,
Thumbnail tinytext NOT NULL, -- References a File DB url
CreationDate date NOT NULL,
LastEditDate timestamp NOT NULL, -- SHOULD NOT BE EDITABLE TO USERS!! default is CreationDate.
Format enum(IMAGE, VIDEO) NOT NULL, -- I think I should change this. This does not play well with everything else. Maybe just set the format based on the contents of the urls? Whether they're .png or .mov or anything?
PostedBy int FOREIGN KEY REFERENCES User(ID) -- Nullable. Null means it's from the static site to webapp import.

```

The Tags database will be another D1 table, this one exclusively holding the tags of each art piece.

The File database will be a R2 Cloudflare database. They allow up to 10GB for free per month which is very nice.

TODO!

#### Considerations

- What's the max length we expect a title to be? It shouldn't be too long for useability. Right now it's 255 just for the sake of it.
- How can we make sure this db will handle emojis and special characters appropriately?

## Creators

This is a D1 SQLite table representing the authors, artists, etc who contribute to the site.
