CREATE TYPE quote_association AS ENUM ('character_index', 'homepage', 'sex_joke', 'quote');

CREATE TABLE quote (
    line text,
    association quote_association
);

INSERT INTO quote(line, association)
SELECT joke, 'character_index'
FROM unnest(ARRAY[
    'Heh. Women.',
    'Now with 25% more Nathaniel.',
    'Shark therians not allowed.',
    'Fun drinking game: take a shot for every mary sue on this list. Have 911 on speed dial.',
    'There he is! My favorite white boy!',
    'DORA THE EXPLORER!?',
    'If you look closely you can actually find Waldo.',
    'Somehow still not passing the Bechdel test.',
    'Everyone on this list have some sort of a police record. Especially the cops.',
    'Fun Fact: this project used to be a self-insert VN for a discord server.',
    'The fact we haven''t been sued by the X-Men writers bewilders us.',
    'AKA The Children of Purity''s hitlist.',
    '& Knuckles',
    'Guess who''s the main character.',
    'Eveyone here needs a therapist.',
    'Who''s that Pokemon?!',
    'First one to find a character here with a tvtropes page that doesn''t apply to it WINS Power Down!',
    'And now, for real, actual facts about Doctor Seanen Goldman!'
    ]) AS joke;

INSERT INTO quote(line, association)
SELECT joke, 'homepage'
FROM unnest(ARRAY[
    'Because discord will not last forever, but random web forums from the 1980s will outlast us all.',
    'Discord? More like Pisscorp.',
    'The real Power Down was the HTML we cried over along the way.',
    'What do you mean &lt;body&gt; has a 8 px margin around itself by default?? WHY??',
    'I built an entire website before I unboxed an Unusual hat.',
    'I want to believe.',
    'Do NOT ask about the cancer healing vagina.', 
    'Funniest button coming Soon(™).',
    'We''re still fishing the orbeez out of the css styling.',
    'We have two kinds of OCs - mary sues, and boring ones.',
    'Found a bug? WHERE?! OH GOD OH FUCK—',
    'Fuck you, baltimore!',
    'We''ll set up an IRC server just as soon as we find an IRC client that doesn''t look like ass.'
    ]) AS joke;

INSERT INTO quote(line, association)
SELECT joke, 'sex_joke'
FROM unnest(ARRAY[
    'Did you know nudism used to be a sign of heroism in ancient Greece? You''re being very brave right now.',
    'Routine forearm exercises.',
    'Time, Dr. Freeman? Is it really that time again?',
    'In hindsight, it''s a good thing we set this in a college.',
    'Boobs, Dicks, Ass. You want it? It''s yours, my friend!',
    'No, you still can''t ask about the Cancer Healing Vagina.',
    'Good thing we''re not keeping count.',
    'Christian-friendly. But like, Ezekiel 23:20 christian-friendly.',
    'Sex 2 couldn''t get any realer.',
    'I have done nothing but draw tits for the past three days!',
    'Now featuring quantum cup sizes!'
    ]) AS joke;

    
INSERT INTO quote(line, association)
SELECT joke, 'quote'
FROM unnest(ARRAY[
    'Did you know nudism used to be a sign of heroism in ancient Greece? You''re being very brave right now.',
    'Routine forearm exercises.',
    'Time, Dr. Freeman? Is it really that time again?',
    'In hindsight, it''s a good thing we set this in a college.',
    'Boobs, Dicks, Ass. You want it? It''s yours, my friend!',
    'No, you still can''t ask about the Cancer Healing Vagina.',
    'Good thing we''re not keeping count.',
    'Christian-friendly. But like, Ezekiel 23:20 christian-friendly.',
    'Sex 2 couldn''t get any realer.',
    'I have done nothing but draw tits for the past three days!',
    'Now featuring quantum cup sizes!'
    ]) AS joke;