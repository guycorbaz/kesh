-- Story v011-2 — fix catch-22 onboarding fresh-install (Issue #120).
--
-- Marqueur `is_stub` sur `companies` : positionné TRUE quand le bootstrap
-- crée une company placeholder (DB vide) pour permettre la création de
-- l'admin du `.env` (sinon catch-22 : pas d'admin sans company, pas de
-- company sans auth). Le wizard d'onboarding repasse `is_stub=FALSE`
-- quand l'utilisateur renseigne ses vraies coordonnées (`set_coordinates`)
-- ou choisit le path demo (`seed_demo`). Le frontend lit ce flag pour
-- afficher un nudge de renommage non-bloquant.
--
-- Non-breaking (epic H8 / CLAUDE.md migration breaking policy P1) :
-- ADD COLUMN avec DEFAULT → les anciens binaires ignorent la colonne, les
-- rows existantes prennent FALSE. Pas de bump `kesh_version_min_required`.
--
-- ALGORITHM=INSTANT / LOCK=NONE : MariaDB 10.3+ supporte l'ajout instantané
-- d'une colonne NOT NULL avec DEFAULT constant (pattern hérité 8-5a-zero).
ALTER TABLE companies
    ADD COLUMN is_stub BOOLEAN NOT NULL DEFAULT FALSE,
    ALGORITHM=INSTANT, LOCK=NONE;
