# Prompt de validation CR-7 P1 — fiches de l'Epic 37 (multi-devises sans retour)

Première passe complète du cycle CR-7, contexte frais, sur une fiche du worktree
`/home/beat/claude/kesh/.claude/worktrees/kesh-wt37` (branche `epic/37-multi-devises` ; n'écris nulle part ailleurs).

## À lire

- `_bmad-output/planning-artifacts/epic-37-multi-devises.md`, section « CR-7 » (A34, A35, A36 KISS) ; décisions et
  arbitrages antérieurs ;
- la fiche visée, sa « Révision CR-7 » (en tête de 37-1 pour le modèle) et le diff `git diff 69b61f06 27ad02ab -- <fiche>` ;
- les autres fiches 37-* pour les points de contact ; `CLAUDE.md` (*Migration breaking policy*, *Pattern batch*,
  *Review Iteration Rule*).

## Ce que tu cherches

1. Défauts introduits par la révision : prémisse fausse au code, contradiction avec la fiche, l'epic ou une fiche sœur
   (routes, codes, gardes, tests, FR/NFR, registre des routes, audit, i18n, manuels), exigence irréalisable.
2. **Résidus** : toute mention restante du mode par exercice réversible — `currency-mode`, `force`,
   `MULTI_CURRENCY_IN_USE`, `FISCAL_YEAR_HAS_FOREIGN_BALANCES`, `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE`,
   `laterFiscalYear`, `monoCurrencyFiscalYear`, désactivation, archivage forcé (hors Change Logs datés).
3. **Comptabilité et protection** : un exercice antérieur à l'activation (clos, rouvert ou non) reste fermé aux devises ;
   la règle du rang 2 de 37-3a ; la limite écrite (exercice créé dans un trou devant un exercice clos puis rouvert) est
   juste et suffisamment bornée.
4. **Décision de l'orchestrateur D-CR7-1 (KISS, à appliquer dans 37-3a et ses renvois)** : le verrou de `companies` pris
   quand une ligne porte `foreign` est **retiré** — la société ne redevient jamais mono-devise, la devise du compte lue
   en partage suffit ; reprends le texte de concurrence et les tests en conséquence, et vérifie qu'aucun autre verrou
   n'en dépendait (37-1, 37-2, 37-3b, 37-5).
5. Tests manquants ; KISS : signale tout mécanisme encore présent qui n'existait que pour la réversibilité.

## Interdits

Edit seulement, dans la fiche visée (et la seule ligne du récapitulatif de l'epic qui la concerne). Aucun script (ni
heredoc, ni `cat >>`), aucun `cargo`/`npm`, aucun `git` en écriture.

## Rapport (français)

Findings `<story>-CR7-P1-N` (sévérité, lieu, constat avec la ligne lue, correctif) ; axes exercés / non exercés ;
décompte. Phase 2 : corrige ; Change Log « 2026-11-16 — validation CR-7 P1 (<modèle>) : <décompte> » ; inter-fiches
signalés avec le texte proposé.
