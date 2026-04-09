-- Migration : table onboarding_state pour le flux d'onboarding
-- Story 2.2 — Flux d'onboarding Chemin A (Exploration)

CREATE TABLE onboarding_state (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    singleton BOOLEAN NOT NULL DEFAULT TRUE COMMENT 'Sentinelle UNIQUE — garantit une seule row',
    step_completed INT NOT NULL DEFAULT 0
      COMMENT '0=pas commencé, 1=langue choisie, 2=mode choisi, 3=chemin choisi (démo ou prod), 4-10 réservés Chemin B (story 2-3)',
    is_demo BOOLEAN NOT NULL DEFAULT FALSE,
    ui_mode VARCHAR(10) NULL COMMENT 'guided|expert — NULL tant que pas choisi',
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT chk_onboarding_step CHECK (step_completed BETWEEN 0 AND 10),
    CONSTRAINT chk_onboarding_ui_mode CHECK (ui_mode IS NULL OR BINARY ui_mode IN (BINARY 'guided', BINARY 'expert')),
    CONSTRAINT chk_onboarding_singleton CHECK (singleton = TRUE),
    CONSTRAINT uq_onboarding_singleton UNIQUE (singleton)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
