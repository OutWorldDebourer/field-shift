-- Token usage of every Anthropic call (consultant rounds, openings, crop research): measured, never estimated.
CREATE TABLE IF NOT EXISTS api_usage (
  id                 BIGINT       NOT NULL AUTO_INCREMENT PRIMARY KEY,
  created_at         DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  analysis_id        CHAR(36)     NULL,
  kind               VARCHAR(24)  NOT NULL,           -- turn, opening or research
  turn_key           VARCHAR(64)  NULL,               -- groups the rounds of one turn
  round              INT          NOT NULL DEFAULT 0,
  model              VARCHAR(64)  NULL,
  input_tokens       INT UNSIGNED NOT NULL DEFAULT 0, -- uncached input
  output_tokens      INT UNSIGNED NOT NULL DEFAULT 0,
  cache_read_tokens  INT UNSIGNED NOT NULL DEFAULT 0,
  cache_write_tokens INT UNSIGNED NOT NULL DEFAULT 0,
  INDEX (analysis_id),
  INDEX (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
