# Prompt — revue de code P2 **ciblée**, Story 28-4a (remédiation de la revue P1)

*Versionné le 2026-10-13. **Une seule lentille — Sonnet** (et non Haiku : story à risque, et la remédiation touche la
production, B-1). Contexte frais, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-4`, branche `story/28-4`, rebasée sur `origin/main` `c28d0a20` (la 28-1
mergée). **Le seul diff à revoir** : `git diff fdd6f94c 8d73a9a4 -- . ':(exclude)_bmad-output'` — `fdd6f94c` = état
revu en P1 (après rebase) ; `8d73a9a4` = **le commit de remédiation**, seul commit de code depuis. Les commits suivants
ne portent que la fiche, le registre, le registre de sprint et les prompts. Pathspec `':(exclude)…'`, jamais `':!…'`.
Rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/28-4a-review-p1-{B,E,A}.md`. Fiche
`_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` (partie a), Change Log « Revue de code P1 » et
Dev Agent Record (annotation de la revue P1). Règles : `CLAUDE.md` du dépôt — § *La passe ciblée*, § « Propagation
post-patch », § « Recompter ses propres comptes rendus », § « Une passe qui ne déclare pas ses axes exercés ne compte
pas ».

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** Rien hors du dépôt et de `/home/gcorbaz/devel/kesh-gate-logs/` ; rien
sous `~/kesh-test/`.

## La lentille — régression de la remédiation

*La sévérité se déplace vers ce qu'on vient d'écrire.* Cherche, dans ce seul diff, le défaut que la remédiation a
**introduit**, et ce qu'elle prétend avoir fait sans l'avoir fait.

1. **La seule production touchée — B-1** (`crates/kesh-api/src/routes/bank_accounts.rs`, `close_bank_account`) :
   la borne `1000..=9999` est-elle juste au domaine de `DATE` de MariaDB ? posée **avant** toute transaction, après la
   forme ? s'applique-t-elle aussi à la date par défaut (`Utc::now`) sans effet de bord ? le message est-il un `400
   VALIDATION_ERROR`, et la réouverture (sans date) n'est-elle pas touchée ? Un `closedOn` à `"+10000-01-01"` atteint-il
   vraiment le handler (désérialisation de `chrono`), sans quoi le test prouverait la forme et non la borne ?
2. **Les doc-comments touchés** (`kesh-db/src/repositories/bank_imports.rs`, `kesh-api/src/routes/bank_imports.rs`,
   `kesh-api/src/errors.rs`, `kesh-db/src/repositories/bank_accounts.rs` — angle mort « archivé et clos ») disent-ils
   vrai au code ? Une affirmation neuve fausse (ex. : la garde de tête contrôle-t-elle bien un compte d'une autre
   société avant l'`INSERT` ? la réouverture filtre-t-elle bien `archived = FALSE` ?) ?
3. **Les tests neufs ou renforcés prouvent-ils ce qu'ils nomment ?** `annuler_un_rejet_pendant_une_cloture_refuse_par_w30`
   (la sonde `["FROM bank_transactions", "AND bank_account_id = ? AND id = ? FOR UPDATE"]` ne peut-elle voir que la
   lecture verrouillante d'`unreject` ? la clôture par SQL tombe-t-elle bien ENTRE R8 et W30 ? passe-t-il à vide si la
   requête finit avant la sonde ?) ; les trois annulations vers un compte archivé (W25, W28, W29 — le règlement est-il
   bien **par virement**, le compte bien celui du paiement ?) ; `export_global_zip_porte_les_comptes_clos` (les
   motifs `;Clos;` et `;2026-01-31;` sont-ils discriminants, la colonne est-elle la bonne ?) ; T-a28 / T-a29 à deux
   propositions (ordre attendu juste ?) ; les trois couples d'ordre de la clôture ; `etat()` à dix colonnes (l'index
   des traces, `[..9]`, est-il juste partout où il est utilisé ?).
4. **La campagne** (`kesh-gate-logs/28-4a-mutations.log`, section « campagne P1 », et
   `28-4a-mutations-p1-bilan.txt` ; script `scratchpad/28-4a/mutations-28-4a-p1.py`, cité au Dev Agent Record) : chaque
   mutation vise-t-elle le site voulu (occurrences 0 et 1 de la garde de `supplier_invoices.rs` = W28 puis W29 ?) ;
   l'équivalence déclarée de M33-a est-elle vraie ?
5. **Propagation.** Le symptôme de A-M1 (« requête en cours … #651 » attaché à W8 / W27) subsiste-t-il ailleurs —
   fiche, CHANGELOG, doc-comments, prompts ? Celui de A-L2 (« non-archivé ») ? Grep **la valeur**.
6. **Le manuel et le PDF** : la remédiation rend-elle fausse une phrase de `docs/manual/fr/*.tex` ou de son PDF (le
   refus d'une date de clôture, l'export global) ? `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/28-4a-review-p2-ciblee-*-flat.txt`.

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-4a-review-p2-ciblee.md`. Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée ;
toute affirmation de présence ou d'absence appuyée d'un `grep -nF` copié. ⛔ **La liste des axes exercés ET non
exercés** (les six ci-dessus) — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport et des textes aplatis dans `/home/gcorbaz/devel/kesh-gate-logs/` ; aucune
commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag`, `sqlx`,
`cargo` (aucune sous-commande), `rustc`, `npm`, `npx`, `node`, `python`/`python3` (pas même les scripts de mutations),
`perl`, `ruby`, `sed -i`, tout interpréteur, `mariadb`, `mysql`, `docker`, `curl`, `wget`, `gh` en écriture,
`pkill`/`kill`, aucune requête SQL. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `git log`/`show`/`diff`,
`gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
