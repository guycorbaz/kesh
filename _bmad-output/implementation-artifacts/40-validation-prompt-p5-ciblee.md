# Prompt de validation P5 ciblée — fiches de l'Epic 40

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), contexte frais, braquée sur **la dernière remédiation**.
Worktree `<dépôt>-wt40` (branche `epic/40-stripe`) : n'écris nulle part ailleurs. Fiche(s) dans ta
mission.

## Périmètre

- Remédiation à relire : `git diff 98bca548 b5d23154 -- <fiche>` (lecture seule) et l'entrée « validation P4 ciblée » du Change Log.
  Contexte : `epic-40-stripe.md` (G*, C-40-*, sections « Validation P1/P2 », D40-P2-1, D40-P3-1), P40-1, fiches 38-*
  et sœurs 40-*.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec la fiche, l'epic, les fiches
  38-* et 40-* (codes, champs, routes, tests), comptablement juste, testée ?
- Points à vérifier (déjà appliqués en P4, à confirmer) : 40-1 — « Tester » appelle `test()` (soldes, liste) puis
  `account_ref()`, dont l'erreur est attribuée à `account_readable` ; au passage, `lastError` porte `PROVIDER_MODE_CHANGED`,
  `PROVIDER_ACCOUNT_CHANGED` ou `PROVIDER_ACCOUNT_UNREADABLE` (amendement déclaré à l'Écart 6 face à 38-3b AC-4) ;
  40-6 — contrôle `account_readable`, codes lus dans `lastError`, ordre des programmes du factice (passage :
  `/v1/account` d'abord ; « Tester » : balance, balance_transactions, account), tout `sk_` → `secret_key` comme 40-1 AC-4 ;
  40-3 — AC-8 n'excepte que la clé `entryDate`.
- Hors périmètre : le reste des fiches.

## Interdits

Edit seulement ; aucun script (ni heredoc, ni `cat >>`), aucun `cargo`/`npm`, aucun `git` en écriture, aucun réseau.

## Rapport (français)

Findings `<story>-P5-N` (sévérité, lieu, constat, correctif) ; axes exercés / non exercés ; décompte. Phase 2 : corrige ;
Change Log « 2026-11-15 — validation P5 ciblée (<modèle>) : <décompte> » ; inter-fiches signalés. Un rapport par fiche.
