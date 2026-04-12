
-- Make "random" an invalid slug name in art and characters
ALTER TABLE art
  ADD CONSTRAINT art_slug_reserve_random
  CHECK (page_slug NOT IN ('random'));

ALTER TABLE character
  ADD CONSTRAINT character_slug_reserve_random
  CHECK (page_slug NOT IN ('random'));