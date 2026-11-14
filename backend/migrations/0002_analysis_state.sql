-- Crop records as JSON (any crop, from the crops service) and the demo-mode flag per analysis.
ALTER TABLE analyses
  ADD COLUMN IF NOT EXISTS crop_current_record JSON NULL,
  ADD COLUMN IF NOT EXISTS demo_mode TINYINT(1) NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS visitor VARCHAR(64) NULL;
