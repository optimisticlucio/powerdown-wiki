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
    'Ah... Amy Rose in the flesh, or rather... the penguin suit..',
    'That''s the noise that happens when you run over people.',
    'Stop selectively breeding people!',
    'It''s so embarrassing, especially when people come over to my house, like "I am so sorry my huge balls slap against my thighs" This happened to me three times today.',
    'Thanks for rubbing one out, or whatever the fuck they say.',
    'Why did I say this',
    'I love how Gabriel is Schrodinger''s 9/11',
    'And a hell yeah to that, bluey.',
    'You are not seeing my hairy man ass on camera for "the bit." ',
    'There is a car',
    'Racism is cool, but skibidi toilet? can''t have that.',
    'We are like oil and water. I''m like a cloud and he''s like... water.',
    'I have free will and hands that can draw.',
    'There will be pickles under your pillow and they will be moist.',
    'If I''m gonna fuck the grandma, give me the wrinkles!',
    'Look, I don''t wanna be a proponent of casual racism, but... competitive racism, right?',
    '9/10 dentists agree that Clyde is a fucking bitch',
    'Give Twerp some estrogen.',
    'I don''t believe I have to say this, but please do not throw your fetus at other members of the voice chat.',
    'THE TRAINS KEEP KISSING. STOP.',
    'If the Soda isn''t at the Starbucks I ain''t buying',
    'YOU REVERSED THE ENTIRE WEBSITE INSTEAD OF DMING ME???',
    'Fuck you, good citizen!',
    'DO IT NOW, FROG MAN, BEFORE I SEND YOU TO THE SCIENCE CLASS. DO YOU KNOW WHAT THEY DO TO FROGS IN SCIENCE CLASSES??'
    ]) AS joke;