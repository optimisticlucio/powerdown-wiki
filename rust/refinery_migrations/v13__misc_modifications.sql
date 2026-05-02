ALTER TABLE misc 
ADD order_position int NOT NULL;

ALTER TABLE misc
ADD CONSTRAINT misc_no_duplicate_positions; 
        UNIQUE (order_position) DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE misc
ADD id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY;

-- This is in the website, so it should always be here.
INSERT INTO misc (
    title, description, url, order_position, thumbnail
) VALUES (
    'Local Tierlist Maker', 'Start petty arguments with your friends about which characters are the least or most likely to shit the bed. Much fun!',
    '/misc/tierlist', 1,
    '/static/img/misc/malicious-and-evil-scout.jpg'
);