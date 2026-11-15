# Prompt de validation P4 ciblée — fiches de l'Epic 40

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), contexte frais, braquée sur **la dernière remédiation**.
Worktree `/home/beat/limu/compta/kesh-wt40` (branche `epic/40-stripe`) : n'écris nulle part ailleurs. Fiche(s) dans ta
mission.

## Périmètre

- Remédiation à relire : `git diff b4dc35dc 88c6b022 -- <fiche>` (lecture seule) et l'entrée « validation P3 » du Change Log.
  Contexte : `epic-40-stripe.md` (G*, C-40-*, sections « Validation P1/P2 », D40-P2-1, D40-P3-1), P40-1, fiches 38-*
  et sœurs 40-*.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec la fiche, l'epic, les fiches
  38-* et 40-* (codes, champs, routes, tests), comptablement juste, testée ?
- **Points inter-fiches à appliquer s'ils visent ta mission** :
  - **Epic** (mission 40-1 seulement) : C-40-3 gagne `PROVIDER_ACCOUNT_UNREADABLE` (`/v1/account` en 403/404, refus à
    « Tester » et au passage, D40-P3-1) ; « DTO des propositions » : « `valueDate` est déjà rendu
    (`transaction.valueDate`) ; gagne `entryDate` seule » ; G12 et FR-40-4-2 : « « Tester » montre `available` et
    `pending` (lus à l'instant, non stockés : l'instantané ne garde que le total, 38-3a) ; l'écran du compte montre le
    solde rapporté total » ; G18 nuancé : « sauf l'attente de 38-9a que 40-4 Écart 6 corrige et la clé `entryDate`
    déclarée par 40-3 ».
  - **40-6** : AC-3 et AC-8 — contrôle `account_readable` dans `provider-labels.ts` (« Cette clé ne peut pas lire le
    compte Stripe : ajoutez le droit de lecture du compte »), code `PROVIDER_ACCOUNT_UNREADABLE` lu dans `lastError`
    comme `PROVIDER_MODE_CHANGED` ; retirer toute mention « passage sans Tester non gardé » ; AC-4 : `valueDate` déjà
    rendu, seule `entryDate` est neuve ; tests, programmes du factice : la réponse de `/v1/account` vient en premier.
- Hors périmètre : le reste des fiches.

## Interdits

Edit seulement ; aucun script (ni heredoc, ni `cat >>`), aucun `cargo`/`npm`, aucun `git` en écriture, aucun réseau.

## Rapport (français)

Findings `<story>-P4-N` (sévérité, lieu, constat, correctif) ; axes exercés / non exercés ; décompte. Phase 2 : corrige ;
Change Log « 2026-11-15 — validation P4 ciblée (<modèle>) : <décompte> » ; inter-fiches signalés. Un rapport par fiche.
