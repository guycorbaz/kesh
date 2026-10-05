# Prompt — revue de code P1, Story 25-4-e (le montant minimum d'une facture)

*Versionné le 2026-10-01. Deux lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-e-montant-minimum-facture`. **Diff à revoir : `git diff 87d022ee 15db5512`** — un seul commit d'implémentation. Fiche :
`_bmad-output/implementation-artifacts/25-4-e-montant-minimum-facture.md` (AC 1–6). CR : `gh issue view 495`. Règles :
`CLAUDE.md` (§ *Migration breaking policy*).

## Lentilles

- **A — Blind + edge-case hunter** : le diff et le code environnant. Comparaison au total arrondi (et si l'arrondi
  est désactivé ?), `<` strict, seuil à 4 décimales stocké mais validé à 2, `Decimal` désérialisé depuis un
  nombre JSON vs une chaîne, `double_option` sur `Decimal`, `null` vs absent, ordre du refus dans
  `validate_invoice` (avant le compte d'arrondi ?), revalidation après dévalidation, avoir, concurrence (le seuil
  lu dans la transaction de validation ?), export CSV, sauvegarde, i18n (arguments Fluent `{ $total }`), frontend
  (champ vide → `null`, valeur invalide → message du serveur affiché ?).
- **C — Acceptance auditor** : chaque AC tenu ? Garde-fous P5/P6/P7/P8 recomptés ; **pars du symptôme** :
  `grep -rn "round_to_5_centimes\|minimum_invoice_amount\|minimumInvoiceAmount" crates/ frontend/src` — un site qui
  énumère les champs des réglages a-t-il été oublié ? Les tests prouvent-ils ce qu'ils disent ? Manuels
  (`docs/manual/fr/{user,admin}-manual.tex` et **PDF aplatis** : `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
  `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`), CHANGELOG.

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu), scénario d'échec,
correction. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un `grep -nF`. ⛔ La liste
des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `cargo check`, `pdftotext` et expériences dans le scratchpad.
