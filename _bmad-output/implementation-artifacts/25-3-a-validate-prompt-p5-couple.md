# Prompt — passe 5 de la 25-3-a-2 et passe 6 (ciblée) de la 25-3-a-1

*Versionné le 2026-09-24. Deux lentilles en contexte frais (Sonnet), cycle Opus → Sonnet.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`, dans
`_bmad-output/implementation-artifacts/` :
- `25-3-a-1-annuler-reglement-client.md` — **validée** en passe 5, puis **rouverte** par le commit
  `18f39bf5` pour y poser une **queue commune** des motifs (AC 3, 9, 11) ;
- `25-3-a-2-annuler-reglement-fournisseur.md` — **réécrite** par le même commit, après sa passe 4
  (9 MEDIUM). Elle s'appuie sur la queue commune.

⛔ Motif mesuré sur ce projet : **la sévérité se déplace vers ce qu'on vient d'écrire.**
⚠️ **Ne conteste pas les arbitrages de Guy** listés dans chaque fiche ; conteste leur **mise en
œuvre**. Un reproche au dépôt de ne pas **encore** faire ce que la fiche prescrit n'est **pas** un
défaut de la spec.

## Lentille A — passe ciblée sur la 25-3-a-1 (seul le diff compte)

Périmètre : `git show 18f39bf5 -- _bmad-output/implementation-artifacts/25-3-a-1-annuler-reglement-client.md`,
puis la fiche entière pour la cohérence.
1. La scission **tête** (rang 1, `settlement_cancel_blocker(settlement_id)`) / **queue**
   (rangs 2-5, `settlement_entry_cancel_blocker(entry_id)`) est-elle implémentable **sans
   deviner**, et cohérente avec la table, l'AC 4 (étape 3), l'AC 8 (champs) et l'AC 11 (tests) ?
2. Le module **partagé** de textes dans `frontend/src/lib/shared/` : vérifie dans
   `frontend/scripts/lint-i18n-ownership.js` qu'il échappe bien au lint, et dans les gardes vitest
   (`frontend/src/lib/shared/i18n-*.test.ts`) qu'un module de `lib/shared/` n'y déclenche pas une
   autre règle (liste d'exemptions, dossier interdit, préfixe global exigé).
3. Reste-t-il une phrase de la fiche qui parle de **l'ancien** module unique dans
   `features/invoices/` ?

## Lentille B — passe complète sur la 25-3-a-2

La fiche entière, comme le développeur qui l'implémentera, **avec** la 25-3-a-1 sous les yeux.
1. **Dépendance** : tout ce que la 25-3-a-2 dit réutiliser existe-t-il bien dans la 25-3-a-1, sous
   ce nom et cette forme ?
2. **L'autorité dans le socle (AC 1)**, la tête (AC 2), le geste (AC 3) : exacts contre le code,
   implémentables, testables sans passer à vide ?
3. **Le risque de double paiement (AC 3)** : le champ `settledByConfirmedBatchId` est-il
   calculable (`payment_batch_items` × `payment_batches.status = 'confirmed'`) ? Une facture peut-elle
   figurer dans **plusieurs** lots confirmés (cycles répétés) — et alors lequel ?
4. **L'écran (AC 7)** contre `supplier-invoices/[id]/+page.svelte` réel ; **les textes (AC 8)** ;
   **les tests (AC 9)** ; **la documentation (AC 10)** — `.tex` **et** PDF aplati
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`).
5. **Exactitude** de chaque `fichier:ligne` neuf.

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), endroit exact, **preuve**, correction.
- ⛔ **La liste des axes exercés ET non exercés.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`,
`npm run`. Lecture, `grep`, `git show`/`log`/`diff`, `pdftotext` (vers le scratchpad) autorisés.
