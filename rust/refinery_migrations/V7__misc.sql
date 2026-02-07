CREATE TABLE misc (
    /*  Because the misc page is barebones as to allow pretty much anything we can't file elsewhere,
        try not to assume all that much about the entries here. */

    title text NOT NULL,
    description text NOT NULL,
    thumbnail text, 
    url text NOT NULL
);