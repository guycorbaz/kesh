# Prompt — passe ciblée P5 de la validation, Story 30-2 (des documents à traiter à la facture fournisseur)

*Versionné le 2026-10-15. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, contexte frais,
**Haiku**, braquée sur le **seul diff de la remédiation P4**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`. Le diff à relire, et **lui seul** :

```sh
git diff 6e8d1266 fd781d07 -- _bmad-output/implementation-artifacts/
```

Il touche `30-2-documents-factures-fournisseur.md` (la fiche), `epic-30-choix-autonomes-302.md` (le registre) et
`sprint-status.yaml`. Rapports qui l'ont motivé : `/home/gcorbaz/devel/kesh-gate-logs/302-validate-p4-R.md` et
`302-validate-p4-F.md` ; la remédiation est l'entrée « Validation P4 » du Change Log de la fiche (tableau, une ligne par
finding). Règles : `CLAUDE.md` (§ *La passe ciblée*, § *Une passe qui ne déclare pas ses axes exercés ne compte pas*,
§ *Haiku-specific guardrails*) ; `docs/agents-orchestration.md` § 6.

⚠️ **Lis le diff aplati ci-dessus, pas une suite de commits** : un seul commit, `fd781d07`. Ce que la fiche disait
avant ce commit n'est pas l'objet de la passe.

## Lentille T — régression de la remédiation P4

Pour **chaque ligne ajoutée** du diff qui affirme un fait du dépôt (chemin, numéro de ligne, nom de fonction, signature,
comportement, clé i18n, code d'erreur), vérifie-le au code par `grep -nF` ou `sed -n` et **copie la sortie**. Cherche
aussi ce que le patch a pu casser autour de lui. Axes, tous à exercer :

1. **La bascule d'AC19** : `bank_accounts::acquire_company_sentinel_lock` existe-t-il à `crates/kesh-db/src/repositories/bank_accounts.rs:1613` et prend-il bien `companies … FOR UPDATE` ? `projects.rs:101` et `supplier_invoices.rs:361-362`, `:724`, `:1071`, `:1314`, `:291-293` disent-ils ce que le tableau des chemins leur fait dire ? L'ordre « staging → facture → `companies` » contredit-il une autre phrase de la fiche (M-c7, « aucun cycle », AC24) ?
2. **Le code `QR_AMOUNT_OUT_OF_RANGE`** : apparaît-il de façon cohérente partout où la fiche liste les codes `qrError` (AC2, AC8, AC9, mutations, `api-external.md`) ? La clé `imported-supplier-invoices-error-qr-amount-out-of-range` et la clé `-qr-unreadable-hint` sont-elles nommées de la même façon à chaque occurrence ? `error-label.ts:45` est-il bien le repli « Échec de l'import » ? Le montant vide (`parser.rs:140-141`) est-il traité comme dit ?
3. **Les sondes d'AC24** : `attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs:561`) et `sonde_verrou_nowait` (l. 611) existent-elles avec les signatures que la fiche suppose ? Reste-t-il dans la fiche **vivante** (hors entrées datées du Change Log) une mention de `300 ms`, de `is_finished` ou de `INNODB_TRX` employée comme moyen de test ?
4. **L'ensemble de blancs d'AC12-bis** : la liste (U+0020, U+0009, U+00A0, U+202F, U+0085, U+200B, U+FEFF) est-elle la même dans la règle serveur, la règle écran et le test prescrits ? Les expressions écrites (`c.is_whitespace() || …`, `/[\s\u0085​]/g`) couvrent-elles exactement cette liste ?
5. **Les autres lignes du tableau P4** (F2, F6, F7, F8, R3, R4, R6 à R15) : chaque remède est-il dans le diff, juste au code (`inbox_import_e2e.rs:252-258`, `:302-305`, `:309` ; `inbox_import.rs:613` ; `imported_supplier_invoices.rs:183-187`, `:247-258` ; `csp.rs:30` ; `iban.rs:11-12` ; `qr_iban.rs:39`), et sans contradiction avec le reste de la fiche ? E6 sur `spc_e2e_discard.png` : reste-t-il une phrase qui donne à E6 `spc_e2e_invoice.png` ?
6. **Recomptes** : 28 AC (`grep -c '^\*\*AC[0-9]'`), 23 tâches (`grep -c '^- \[ \] \*\*T-'`), 39 mutations (`grep -cE '^\| M-[abc][0-9]+ '` ; M-a13 absente, M-a15 présente), trois déclarées équivalentes (M-a14, M-b2, M-c7), neuf parcours E2E, 23 choix — et `T-a9` dit-il la bonne plage de mutations ?
7. **Entrées datées** : le diff a-t-il **réécrit** une entrée passée du Change Log ou du registre (interdit) au lieu de l'**annoter** ?
8. **Manuel** : le diff prescrit-il d'écrire au manuel (`docs/manual/fr/user-manual.tex`, `admin-manual.tex`) quelque chose que le code prescrit ne tiendrait pas (mention « accepté sans les données du QR », codes `qrError`, bascule) ? Contrôle aussi le PDF si tu cites une phrase existante, **aplati vers la sortie standard seulement** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.

**Haiku — garde-fous** (`CLAUDE.md`) : tout finding qui affirme l'**absence** d'un patch ou la **présence** d'un
anti-pattern s'appuie sur une sortie `grep -nF` copiée ; un livrable que la fiche **prescrit** au développement (code,
test, clé i18n) n'existe pas encore dans le dépôt — son absence n'est **pas** un défaut ; ne signale que ce que la fiche
**affirme** du dépôt actuel et qui est faux, ou ce qu'elle se contredit.

## Ce que tu rends

Findings numérotés `T1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, **ligne de la fiche** (ou du registre) et
**ligne du code** concernées, **preuve** (commande exécutée et sortie copiée), correction proposée. ⛔ **La liste des axes
exercés ET non exercés** (1 à 8 ci-dessus) — un « 0 finding » sans elle ne compte pas comme passe. Rapport **écrit d'un
seul jet**, un seul fichier, à la fin, sans brouillon ni fichier intermédiaire :
`/home/gcorbaz/devel/kesh-gate-logs/302-validate-p5-T.md`. Dernier message : le chemin du rapport, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport. Lecture seule stricte : **aucun `git checkout`, `git switch`,
`git checkout --detach` ni aucune autre forme** ; aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`),
`make`, `latexmk`, `xelatex`, `git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`,
`npx`, `node`, **`python3`, `python`, `perl`, `ruby` et tout autre interpréteur**, `docker`, **`mariadb`, `mysql` et
toute requête SQL**, `bash` sur un script du dépôt, `gh` en écriture. `pdftotext` vers la **sortie standard** seulement,
jamais vers un fichier. Aucune donnée réelle de Guy ne se lit ni ne se cite (rien de `~/kesh-test/`). Autorisés :
lecture, `grep`, `sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`, `pdftotext … -`.
