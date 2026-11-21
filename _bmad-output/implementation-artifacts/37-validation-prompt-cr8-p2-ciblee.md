# Prompt de validation CR-8 P2 ciblée — fiche 37-5 (contrôle de clôture bloquant)

Passe ciblée (modèle différent de P1, Opus) braquée sur la remédiation P1 :, contexte frais. Worktree `<dépôt>/.claude/worktrees/kesh-wt37`
(branche `epic/37-multi-devises`) : n'écris nulle part ailleurs.

## À lire

- epic `_bmad-output/planning-artifacts/epic-37-multi-devises.md`, sections « CR-7 » et « CR-8 » (A37–A40) et la
  précision du responsable de l'epic (l'avis K9 montre l'aperçu avant « Réévaluer » ; rien n'est présenté quand aucune réévaluation n'est
  nécessaire) ; KISS (A36) ;
- `37-5-reevaluation-fin-exercice.md`, son AC-9 neuf et le diff `git diff 41c3e568 e4782370 -- <fiche> (remédiation P1 : close sans changement de signature, cas du 31.12.2024, blockers affichés, postes non monétaires décochés)` ;
- le code des contrôles de clôture (`closing_checks.rs`, `fiscal_years::close`, `routes/fiscal_years.rs`, écran
  *Exercices*, `FiscalYearChecks.svelte`) et les fiches sœurs 37-1 à 37-4.

## Ce que tu cherches

1. Prémisses fausses au code (chaque ligne citée), contradictions avec la fiche, l'epic, les fiches sœurs ; exigence
   irréalisable.
2. **Le blocage** : refus réel dans `fiscal_years::close` (étape d-bis, 409 `FOREIGN_ACCOUNTS_NOT_REVALUED`), ordre des
   refus, verrous et lecture des soldes après les verrous, rejeu ; une seule fonction de calcul (aperçu, passation, K9,
   clôture) ; pas de lecture coûteuse quand rien n'est à réévaluer.
3. **Comptabilité** : le résidu juge-t-il bien au cours par défaut ? Cas des livres importés passés à un autre cours
   (`rates`, V8) : K9 reste en attention — est-ce le comportement voulu, et le remède (saisir le cours à l'écran *Cours
   de change*) est-il juste et écrit ? Pour le 31.12.2024 du responsable de l'epic (cours journaliers AFC 0.91193 / 0.94947, genre de
   clôture `day`), K9 passe-t-il sans saisie ?
4. **Écran** : aperçu dans l'avis avant clic, rien quand OK, « Clôturer » désactivé, Consultation, liens (grand livre,
   écriture), champs ajoutés à l'aperçu (additifs).
5. **Inventaire « huit contrôles » → neuf** : complet ? (grep `huit`, `toHaveCount(8)`, `checks[`, `Ces contrôles ne
   bloquent rien`) ; `.tex` seulement ; Note de version.
6. Tests manquants ; KISS.

## Interdits

Edit seulement, dans 37-5 (et sa ligne du récapitulatif de l'epic). Aucun script (ni heredoc, ni python, ni `>>`),
aucun `cargo`/`npm`, aucun `git` en écriture.

## Rapport (français)

Findings `37-5-CR8-P2-N` ; axes exercés / non exercés ; décompte. Phase 2 : corrige ; Change Log « 2026-11-16 —
validation CR-8 P2 ciblée (<modèle>) : <décompte> » ; inter-fiches signalés.
