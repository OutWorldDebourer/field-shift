-- Field Shift initial schema (MariaDB 11.4). Spatial column for the field outline.

CREATE TABLE IF NOT EXISTS analyses (
  id            CHAR(36)     NOT NULL PRIMARY KEY,
  location_mode VARCHAR(16)  NOT NULL DEFAULT 'example',   -- 'gps' | 'example' | 'map'
  example_site  VARCHAR(64)  NULL,
  lang          VARCHAR(2)   NOT NULL DEFAULT 'en',
  crop_current  VARCHAR(128) NULL,
  crop_history  JSON         NULL,
  priorities    JSON         NULL,
  share_token   CHAR(22)     NULL UNIQUE,
  created_at    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS fields (
  id          BIGINT       NOT NULL AUTO_INCREMENT PRIMARY KEY,
  analysis_id CHAR(36)     NOT NULL,
  center_lat  DOUBLE       NOT NULL,
  center_lon  DOUBLE       NOT NULL,
  outline     POLYGON      NOT NULL,
  area_ha     DOUBLE       NOT NULL DEFAULT 0,
  label       VARCHAR(128) NULL,
  created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  SPATIAL INDEX (outline),
  CONSTRAINT fk_fields_analysis FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS messages (
  id          BIGINT      NOT NULL AUTO_INCREMENT PRIMARY KEY,
  analysis_id CHAR(36)    NOT NULL,
  role        VARCHAR(16) NOT NULL,                       -- 'user' | 'assistant' | 'status'
  body        MEDIUMTEXT  NOT NULL,
  meta        JSON        NULL,
  created_at  DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX (analysis_id, id),
  CONSTRAINT fk_messages_analysis FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS photos (
  id          BIGINT       NOT NULL AUTO_INCREMENT PRIMARY KEY,
  analysis_id CHAR(36)     NOT NULL,
  message_id  BIGINT       NULL,
  file_path   VARCHAR(255) NOT NULL,                      -- relative to the uploads folder; files live outside the DB
  mime        VARCHAR(64)  NOT NULL,
  bytes       INT          NOT NULL,
  note        VARCHAR(255) NULL,
  created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX (analysis_id),
  CONSTRAINT fk_photos_analysis FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS sections (
  analysis_id CHAR(36)    NOT NULL,
  section     VARCHAR(32) NOT NULL,                       -- field | soil | pests | nutrition | rotation | sheet
  state       VARCHAR(16) NOT NULL DEFAULT 'empty',       -- empty | progress | ready | gaps
  content     JSON        NULL,
  updated_at  DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (analysis_id, section),
  CONSTRAINT fk_sections_analysis FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS reports (
  id          BIGINT   NOT NULL AUTO_INCREMENT PRIMARY KEY,
  analysis_id CHAR(36) NOT NULL,
  evidence    JSON     NOT NULL,
  created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX (analysis_id),
  CONSTRAINT fk_reports_analysis FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS source_cache (
  cache_key  VARCHAR(255) NOT NULL PRIMARY KEY,
  source     VARCHAR(64)  NOT NULL,
  payload    LONGTEXT     NOT NULL,
  fetched_at DATETIME     NOT NULL,
  INDEX (source)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS crop_traits (
  crop_key   VARCHAR(128) NOT NULL PRIMARY KEY,           -- normalized lowercase scientific or common name
  record     JSON         NOT NULL,                       -- crop record + traits, every value with its citation
  origin     VARCHAR(16)  NOT NULL,                       -- 'seed' | 'ai-researched'
  created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS rate_limits (
  bucket     VARCHAR(96) NOT NULL PRIMARY KEY,            -- hashed visitor + window
  hits       INT         NOT NULL DEFAULT 0,
  window_end DATETIME    NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
