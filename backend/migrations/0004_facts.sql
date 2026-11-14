-- What the farmer has told so far (intake form and consultant notes), as one JSON object.
ALTER TABLE analyses
  ADD COLUMN IF NOT EXISTS facts JSON NULL;
