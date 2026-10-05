# Prompt — revue de code P3 ciblée, Story 25-4-d2a (solder le reste)

*Versionné le 2026-10-02. Une lentille (Sonnet), contexte frais — passe ciblée sur la remédiation de P2. Trend : P1 1C/6M/4L
(Sonnet ×3) → P2 1M/4L (Opus), dont le MEDIUM était une régression du correctif de P1. **La sévérité se déplace vers ce
qu'on vient d'écrire** : cherche d'abord ce que ce dernier patch a cassé.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2a-ecriture-de-solde`. **Diff à revoir : `git diff abbf1bdc 8a2537d9`**
(un commit). Fiche : `_bmad-output/implementation-artifacts/25-4-d2a-ecriture-de-solde.md` (Change Log, entrées « Revue de
code P1 » et « P2 »).

## Lentille — regression hunter

La remédiation :
1. dans `write_off_journal_lines` (`crates/kesh-db/src/repositories/invoice_settlements.rs`), quand le débit de la nature
   vaut zéro **et** que la fraction de centime est séparée, la ligne de nature est omise : tout le reste va au compte
   d'arrondi ; la garde devient `nature_debit < 0 || (nature_debit == 0 && !separate_gap)` ;
2. dans `write_off_invoice` (`invoice_settlements_write.rs`), le compte d'arrondi est désormais verrouillé **avant** le
   compte de TVA ;
3. trois tests ajoutés ou étendus.

Vérifie, **en recalculant les montants** :
- toutes les combinaisons de la nouvelle garde : `nature_debit` négatif, nul avec et sans écart séparé, positif ; avec
  et sans TVA ; nature `rounding` ; comptes de nature et d'arrondi identiques pour une nature autre que `rounding`. Une
  écriture d'**une seule ligne de débit** sur le compte d'arrondi est-elle acceptée par `journal_entries::create_in_tx`
  (nombre minimal de lignes, comptes distincts exigés ?) — lis sa validation ;
- l'ordre des verrous : énumère, dans `write_off_invoice`, chaque `FOR UPDATE` dans l'ordre réel, et compare-le à celui
  de la validation d'une facture arrondie (`invoices.rs`), de l'avoir (`credit_notes.rs`), du règlement au centime
  (`settle_invoice`) et de l'acceptation de rapprochement. Un cycle subsiste-t-il ?
- le libellé « Solde facture … — frais bancaires » d'une écriture qui ne touche plus que le compte d'arrondi : est-ce
  trompeur au grand livre, et l'audit dit-il vrai (nature, montant, TVA) ?
- les trois tests : chacun aurait-il échoué avant ce patch ? Prouvent-ils ce qu'ils disent ?

## Ce que tu rends

Findings avec sévérité, `fichier:ligne`, **preuve** (calcul posé, commande et sortie). Pour tout finding affirmant qu'un
code est absent ou présent : la sortie d'un `grep -nF`. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `cargo check`, expériences dans le scratchpad.
