CREATE TYPE quote_association AS ENUM ('character_index');

CREATE TABLE quotes (
    line text,
    association quote_association
);

INSERT INTO quotes(line, association)
SELECT joke, 'character_index'
FROM unnest(ARRAY[
    'AKA The Children of Purity''s hitlist.',
    'Somehow still not passing the Bechdel test.',
    'Everyone on this list have some sort of a police record. Especially the cops.'
    ]) AS joke;