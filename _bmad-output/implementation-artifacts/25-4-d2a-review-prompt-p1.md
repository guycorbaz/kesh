# Prompt — revue de code P1, Story 25-4-d2a (solder le reste : l'écriture et son annulation)

*Versionné le 2026-10-01. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2a-ecriture-de-solde`. **Diff à revoir : `git diff f6eff8d3 33b4f201`** —
un seul commit d'implémentation. Fiche : `_bmad-output/implementation-artifacts/25-4-d2a-ecriture-de-solde.md` (AC 1–8,
Dev Agent Record). Issues : `gh issue view 384`, `gh issue view 490`, `gh issue view 491`. Règles : `CLAUDE.md`
(§ *Migration breaking policy*, § *Review Iteration Rule*, § *Pattern batch*).

## Lentilles

- **A — Blind hunter** : le diff **seul**, sans la fiche. Ce qui est faux, fragile ou dangereux dans le code tel qu'il
  est écrit : équilibre de l'écriture, signes, arrondis, ordre des verrous, gestion d'erreur, SQL, types (`Decimal`,
  JSON), réactivité frontend.
- **B — Edge-case hunter** : le diff et le code environnant.
  - Comptabilité : une facture à **plusieurs taux** avec lignes à 0 % et arrondi figé négatif ; un reste à quatre
    décimales ; la TVA corrigée d'un escompte sur une facture **déjà partiellement réglée** ; le compte de la nature
    **égal** au compte de TVA ou à la créance ; un compte de TVA due **archivé** (lu sans contrôle d'activité ?) ; le
    compte de TVA due courant différent de celui de l'écriture de vente.
  - Concurrence : `write_off_invoice` contre un règlement manuel, une acceptation de rapprochement, une annulation de
    règlement, un avoir — chaque paire. Le rejeu `retry_with` : que se passe-t-il au deuxième essai quand la `version`
    a été incrémentée par la tentative qui a échoué (elle a été annulée par le 1213 — vérifie) ?
  - L'invariant « tant qu'un solde existe, la facture est payée » : **énumère tous les écrivains** de `paid_at` et de
    `invoice_settlements` (`grep -rn "paid_at\s*=\|DELETE FROM invoice_settlements\|INSERT INTO invoice_settlements" crates/ --include=*.rs`)
    et dis pour chacun s'il peut le casser (dévalidation, avoir, rapprochement, annulation, import, rejeu post-restore).
  - Le motif `WriteOffExists` : lecture et écriture disent-elles la même chose ? Le dé-rapprochement le rencontre-t-il
    réellement (lis `reconciliation_cancel.rs`) ? Le frontend du dé-rapprochement l'affiche-t-il ?
  - Sauvegarde et restauration : un `.keshbackup` **récent** (avec soldes) réimporté — la colonne JSON, les CHECK, le
    registre `POST_RESTORE_BACKFILLS` (la migration `20260828000001` rejoue un backfill de classe B sur
    `invoice_settlements` : touche-t-il une ligne `write_off` ?).
- **C — Acceptance auditor** : chaque AC tenu ? Garde-fous P5/P6/P7/P8 **recomptés depuis la source**. **Pars du
  symptôme** : tout site qui lit `settlement_type` ou énumère les colonnes de `invoice_settlements`
  (`grep -rn "settlement_type\|settlement_account_id" crates/ frontend/src`) traite-t-il `write_off` ? Les tests
  prouvent-ils ce qu'ils disent (le test du rang, le test #490, le test d'import sans colonnes) ? `sitesTotal` et les
  décomptes du registre d'audit recomptés ? `docs/api-external.md` dit-il vrai (codes et statuts contre `errors.rs`) ?
  CHANGELOG. Le Dev Agent Record n'affirme-t-il que ce qui a tourné ? Le manuel (`docs/manual/fr/*.tex` et **PDF
  aplatis**, `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
  `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) dit-il quelque chose
  que cette story rend faux (annulation d'un règlement, motifs de refus, reste insoldable de #490) ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu), scénario d'échec,
correction. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF`. ⛔ La liste des
axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `cargo check`, `pdftotext` et expériences dans le scratchpad.
