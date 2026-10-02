# Prompt — revue de code P1, Story 25-4-d2c (le rapport TVA retranche la TVA des soldes)

*Versionné le 2026-10-02. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2c-rapport-tva-soldes`. **Diff à revoir : `git diff ff116692 043e8209`** —
un seul commit d'implémentation. Fiche : `_bmad-output/implementation-artifacts/25-4-d2c-rapport-tva-soldes.md`
(AC 1–8, limites, Dev Agent Record). Contexte : le solde est écrit par `write_off_invoice`
(`crates/kesh-db/src/repositories/invoice_settlements_write.rs`), ses lignes d'écriture par `write_off_journal_lines`
(`crates/kesh-db/src/repositories/invoice_settlements.rs`), sa ventilation par `write_off_vat_shares`
(`crates/kesh-core/src/accounting/vat.rs`). Issues : `gh issue view 384`, `390`. Règles : `CLAUDE.md`.

## Lentilles

- **A — Blind hunter** : le diff **seul**, sans la fiche. Ce qui est faux ou fragile : SQL (filtres société, période,
  type, jointures qui dupliquent des lignes, `SUM` sur ensemble vide → `NULL`), arithmétique `Decimal` (signe, arrondi,
  ordre d'agrégation), gestion d'erreur, réactivité Svelte 5, `big.js` (aucun `Number` sur un montant), typage.
- **B — Edge-case hunter** : le diff et le code environnant.
  - **Le delta** : refais le calcul pour (a) facture à deux taux + escompte dans la période ; (b) facture d'une période
    antérieure, solde dans la période ; (c) solde de frais bancaires ou d'arrondi (`[]`) ; (d) solde dont la part
    s'arrondit à 0.00 ; (e) solde avec fraction de centime (le compte d'arrondi reçoit-il une ligne qui touche le compte
    de TVA due ?) ; (f) solde annulé (la contre-passation touche-t-elle le compte de TVA due **dans la période**, et
    sort-elle du filtre `INNER JOIN invoice_settlements` ? la ligne de règlement est-elle supprimée ou marquée ?) ;
    (g) une facture **avoir** ; (h) plusieurs soldes sur plusieurs factures, même taux écrit `8.1` et `8.10`
    (l'agrégation par taux fusionne-t-elle ?). Le delta reste-t-il nul là où la fiche l'affirme ?
  - **La cohérence avec `rows`** : les taux des soldes sont-ils comparables aux taux des lignes de vente (même échelle,
    même tri) ? Une ligne de solde à un taux absent des ventes s'affiche-t-elle ?
  - **Les rendus** : CSV (colonnes, échappement, ligne de titre), PDF (débordement de page, libellés), écran (états :
    chargement, rapport vide, soldes sans vente, `writeOffRows` absent d'une ancienne réponse) — et les trois gardes
    « vide ».
  - **`CorruptData`** : toutes les formes fausses mènent-elles à l'erreur (tableau d'objets vide ? `NULL` en base ?
    nombre au lieu de chaîne) ? Le détail interne fuit-il dans la réponse HTTP ?
- **C — Acceptance auditor** : chaque AC tenu ? `sitesTotal`, clés i18n dans les 4 locales (parité, replis identiques au
  FTL fr-CH) — recomptés depuis la source. **Pars du symptôme** : `grep -rn "vat_balance\|vatBalance\|total_vat_due\b\|totalVatDue\b" crates frontend/src`
  — un consommateur de `vat_balance` ou de `total_vat_due` qui suppose encore l'ancien sens (brut) ? Les tests
  prouvent-ils ce qu'ils disent (la mutation que chacun tue est-elle réellement tuée par l'assertion écrite) ? Le
  **manuel** (`docs/manual/fr/user-manual.tex` et **PDF aplati**, `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
  `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) dit-il ce que fait le
  code — section *Décompte TVA*, section des soldes, glossaire, limites (solde annulé, parts sans TVA) ? Le CHANGELOG ?
  Le Dev Agent Record n'affirme-t-il que ce qui a tourné (décomptes de tests aux deux bornes) ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu), scénario d'échec,
correction. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF`. ⛔ La liste des
axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` et expériences dans le scratchpad.
