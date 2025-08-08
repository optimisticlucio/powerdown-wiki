# Code Structure

A proposed code structure for the rust code that's going to be running the backend.

This code's going to be working on cloudflare's infrastructure, including

- **Workers**, which spin up an instance of the code per request. (Meaning I probably should avoid in-memory caching or anything that relies on a given instance being spun up)
- **D1**, an SQLite database
- **R2**, a persistent storage database

----

## Code Graph

```mermaid
graph LR

main>"main.rs<br/>Entry Point"]

lib["lib.rs"]

index["index.rs<hr/>Handles root pages like index or 404"]

subgraph Admin
    admin["admin.rs"]
end

subgraph Art Archive
    art["art.rs"]
    art-page["/art/page.rs<hr/>For displaying individual art pages"]
    art-search["/art/search.rs<hr/>Displaying the art selection page, with possible filters."]

    art -- "/{art slug}" --> art-page
    art -- "/" --> art-search
end

subgraph Characters
    char["characters.rs"]
end

subgraph Lore
    lore["lore.rs"]
end

subgraph Recaps
    recaps["recaps.rs"]
end

subgraph Stories
    stories["stories.rs"]
end

static["static_files.rs"]
misc["misc.rs"]
search["search.rs"]

main ---> lib

lib -- "/" --> index
lib -- "/art-archive/" --> art
lib -- "/characters/" --> char
lib -- "/stories/" --> stories
lib -- "/lore/" --> lore
lib -- "/recaps/" --> recaps
lib -- "/static/" --> static
lib -- "/misc/" --> misc
lib -- "/admin/" --> admin
lib -- "/search/" --> search

```

### Considerations

- How do I handle NSFW art index? Is it its own url, or just a cookie?
