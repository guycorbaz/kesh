# Prompt — validation P2, Story 25-6-b (le PDF d'une facture est figé)

*Versionné le 2026-10-04. Une passe (Opus), contexte frais, les trois lentilles. Trend : P1 4 HIGH / ~15 MED (Sonnet ×3).
La remédiation ajoute une route (refiger) et réécrit plusieurs AC : protocole complet, attention particulière au diff.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-b-pdf-facture-archive`. **Fiche :**
`_bmad-output/implementation-artifacts/25-6-b-pdf-facture-archive.md` (tête `5ed8d632`). Remédiation de P1 :
`git diff cd4a3275 5ed8d632`. Issue `gh issue view 387` ; CR `502`, `503`. Règles : `CLAUDE.md` (§ *Migration breaking
policy* P1 à P8 ; § *Pattern batch* sans objet). Les **huit arbitrages** sont rendus par Guy : un finding qui en conteste
un doit dire pourquoi il serait faux, pas qu'un autre choix existe.

## Ce que tu vérifies

1. **La remédiation de P1** :
   - la garde `status = 'validated'` sur la pose : ferme-t-elle vraiment la course gel/dévalidation (lis `unvalidate`,
     `crates/kesh-db/src/repositories/invoices.rs:1441-1610`, et l'avoir, `credit_notes.rs:583`) ? Et la course
     gel/**avoir** — un avoir qui passe la facture en `cancelled` entre le rendu et la pose : le document n'est-il alors
     jamais figé, et est-ce le bon comportement ?
   - **refiger (AC 3-bis)** : la route dans le bloc d'administration (`crates/kesh-api/src/lib.rs`, marqueur
     `KESH-ADMIN-ROUTES-END`, `require_admin_role`, `require_not_pat`) ; ses conditions (fichier absent / présent /
     altéré / non figé / `cancelled`) sont-elles complètes et sans contradiction avec AC 2, AC 3 ? La garde
     `pdf_sha256 = <ancienne>` ; le registre des routes d'audit (`crates/kesh-api/tests/audit_route_registry.rs`) impose-t-il
     d'y inscrire la route ? Le refigeage d'une facture dont le **client a changé de langue** ;
   - l'audit dans la transaction de la pose (`audit_log::insert_in_tx`) et l'acteur « clé d'API » : l'audit sait-il
     nommer une clé d'API aujourd'hui (`grep -rn "api_key_id" crates/kesh-db/src/repositories/audit_log.rs`) ?
   - la constante unique de colonnes (AC 1-bis) : réalisable sans casser `INVOICE_COLUMNS` de `reconciliation.rs`, qui
     préfixe peut-être par un alias de table ?
   - l'export CSV (AC 5-bis) et son test d'en-tête.
2. **Cohérence interne** : la fiche dit-elle la même chose partout (arbitrages, AC, tâches, limites, *Modules*, Change
   Log) ? En particulier « jamais régénérer » (Dev Notes) contre « refiger » (arbitrage 7).
3. **Le manuel** : les sites nommés à l'AC 8 existent-ils et disent-ils ce que la fiche affirme (`sed -n` sur
   `docs/manual/fr/user-manual.tex` et `admin-manual.tex`) ? En manque-t-il (`grep -rn "PDF" docs/manual/fr/user-manual.tex`) ?
4. **Périmètre** : toujours quatre modules de code ? Le signal de découpage (§ *Règle de splitting préventif*) est-il
   atteint par l'ajout de la route ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit, **preuve** (commande ET sortie, `grep -nF` pour toute
présence ou absence), correction proposée. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
