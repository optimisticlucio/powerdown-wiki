ALTER TABLE lore
ADD COLUMN category_order_position int NOT NULL DEFAULT 0; -- Zero-indexed ordering of the lore pages within a given category