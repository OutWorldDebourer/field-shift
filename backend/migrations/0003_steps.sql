-- Guided conversation: state of each topic (field → soil → pests → nutrition → rotation → sheet) per analysis.
ALTER TABLE analyses
  ADD COLUMN IF NOT EXISTS steps JSON NULL;
