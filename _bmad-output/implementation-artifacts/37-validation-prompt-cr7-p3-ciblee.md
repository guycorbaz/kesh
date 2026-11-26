# Prompt de validation CR-7 P3 ciblée — fiches de l'Epic 37

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), contexte frais, braquée sur la dernière remédiation. Worktree
`<worktree>/kesh-wt37` (branche `epic/37-multi-devises`) : n'écris nulle part ailleurs.

## Périmètre

- Remédiation à relire : `git diff beb17741 e08908bc -- <fiche>` (lecture seule) et l'entrée « validation CR-7 P2 » du
  Change Log. Contexte : section « CR-7 » de l'epic (A34–A36, D-CR7-1 à D-CR7-3).
- Points à confirmer : 37-1 — refus 400 `LATER_FISCAL_YEAR_MONO_CURRENCY` dans `fiscal_years::create` (lecture de
  `companies.multi_currency_enabled` en partage en tête de transaction, ordre `companies` → exercices, ordre avec
  `LATER_FISCAL_YEAR_CLOSED`, tests), limite assumée retirée partout ; 37-3a — D-CR7-2 cité, phrase de la limite retirée ;
  37-3b — fixtures par les routes (création, clôture, activation, réouverture), cycle S → X sur `companies` des
  conversions avec projet, défendu par le rejeu de la route ; 37-5 — relecture sous verrou du mode retirée (D-CR7-3),
  409 jugé sur l'exercice seul.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec la fiche, l'epic et les
  fiches sœurs ; testée ; aucun résidu du modèle réversible ni de la limite assumée (hors Change Logs datés) ; KISS.
- Hors périmètre : le reste des fiches.

## Interdits

Edit seulement, dans la fiche visée. Aucun script (ni heredoc, ni python, ni `cat >>`), aucun `cargo`/`npm`, aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-CR7-P3-N` (sévérité, lieu, constat, correctif) ; axes exercés / non exercés ; décompte. Phase 2 :
corrige ; Change Log « 2026-11-16 — validation CR-7 P3 ciblée (<modèle>) : <décompte> » ; inter-fiches signalés.
