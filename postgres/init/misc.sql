CREATE TABLE misc (
    /*  Because the misc page is barebones as to allow pretty much anything we can't file elsewhere,
        try not to assume all that much about the entries here. */

    title text NOT NULL,
    description text NOT NULL,
    thumbnail text, 
    url text NOT NULL
);

INSERT INTO misc (
    title, description,
    thumbnail, url
) VALUES (
    'The Ritual', 'A torture session we do to newbies. If you do this, GET ON THE SERVER VC FIRST AND TELL US.',
    'https://powerdown.wiki/assets/img/misc/the_ritual.jpg', '/ritual'
);

INSERT INTO misc (
    title, description,
    thumbnail, url
) VALUES (
    'RP Guidelines', 'Suggestions and assistance for people new to roleplaying.',
    'https://powerdown.wiki/assets/img/misc/rp_guidelines.jpg', '/guides/rp-guidelines'
);
