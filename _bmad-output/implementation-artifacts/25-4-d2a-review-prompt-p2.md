# Prompt — revue de code P2 ciblée, Story 25-4-d2a (solder le reste)

*Versionné le 2026-10-02. Une lentille (Opus), contexte frais — passe ciblée sur la remédiation de P1 (cf. `CLAUDE.md`
§ « La passe ciblée »). La remédiation touche du code de production qui écrit en comptabilité : sois exhaustif sur
elle.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2a-ecriture-de-solde`. **Diff à revoir : `git diff 6a4ac4ff b8663cc1`**
(un commit). Fiche : `_bmad-output/implementation-artifacts/25-4-d2a-ecriture-de-solde.md` (Change Log, entrée « Revue de
code P1 »). La version antérieure de la fonction : `git show 33b4f201:crates/kesh-db/src/repositories/invoice_settlements.rs`.

## Lentille — regression hunter

La remédiation :
1. réécrit `write_off_journal_lines` (`invoice_settlements.rs`) : la fraction de centime d'un reste à quatre décimales
   va au compte de différences d'arrondi, sauf quand le compte de la nature **est** ce compte (nature `rounding`) ;
2. ajoute dans `write_off_invoice` la relecture du compte d'arrondi quand la fraction existe, et
   `vat_payable_account_for_write` (`company_invoice_settings.rs`) pour le compte de TVA ;
3. ajoute cinq tests et corrige un commentaire et un en-tête de registre.

Vérifie, **en recalculant toi-même les montants** :
- l'équilibre de l'écriture dans **tous** les cas : écart positif, négatif, nul ; avec et sans TVA ; nature `rounding`
  avec un reste à quatre décimales **supérieur** au centime (0.0249) ; reste minuscule (0.0040) ; un reste où
  `arrondi(A) − Σ TVA` serait nul ou négatif alors que `A − Σ TVA` est positif ;
- la garde `total_vat >= amount` : compare-t-elle maintenant la bonne grandeur, sachant que le débit de la nature se
  calcule sur l'arrondi ?
- le sens de l'écart (débit si positif) : est-il le **même** que dans `settlement_journal_lines`, ou l'inverse — et
  lequel est juste ? (Dans le règlement, l'écart est `payé − réglé` ; ici, `A − arrondi(A)`.)
- l'ordre des verrous : le compte d'arrondi est verrouillé **après** le compte de la nature et **avant** le compte de
  TVA — un cycle avec un autre écrivain qui verrouille les mêmes comptes dans un autre ordre (règlement au centime,
  validation d'une facture arrondie, acceptation de rapprochement) ?
- `vat_payable_account_for_write` : `postable = TRUE` exigé — le compte de TVA des plans livrés et de la société de test
  l'est-il ? Une société existante dont le compte de TVA ne l'est pas serait-elle soudain bloquée ?
- les cinq tests : chacun aurait-il échoué avant la remédiation ? Le montage du test « arrondi figé » (lignes d'écriture
  modifiées en SQL) reproduit-il fidèlement ce que fait la validation (`generate_invoice_journal_lines_rounded`) ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (calcul posé, commande et sortie). Pour tout finding affirmant qu'un
code est absent ou présent : la sortie d'un `grep -nF`. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `cargo check`, expériences dans le scratchpad.
