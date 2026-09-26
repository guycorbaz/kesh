# Prompt — passe 1 de `bmad-code-review`, Story 25-5-a

*Versionné le 2026-09-26. Trois lentilles en contexte frais (Sonnet), orthogonales à l'auteur du
code (Opus 5). Protocole du workflow : Blind Hunter, Edge Case Hunter, Acceptance Auditor.*

⚠️ **Revue a posteriori** : la story a été **mergée sans revue de code** (PR #452, commit squash
`91a20d76` sur `main`, issue #386 fermée). Sa fiche est restée en `review`. Les correctifs éventuels
iront sur la branche `story/25-5-a-revue-code`.

Dépôt `/home/gcorbaz/devel/kesh`. **Diff revu** : `git diff 91a20d76^ 91a20d76` sur le code et la
documentation (PDF et `_bmad-output/` exclus), enregistré dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/25-5-a.diff`
(2019 lignes). Fiche : `_bmad-output/implementation-artifacts/25-5-a-export-souverainete.md` — ses
AC (1 à 11), son arbitrage (« l'export CSV sert à migrer : seules les données comptables ; le
backup sert à restaurer : tout » — **ne pas le contester**, en contester la mise en œuvre) et son
Dev Agent Record, qui déclare **deux arbitrages tranchés** par le développeur et **un écart** à la
fiche (partition 11 + 9, et non 11 + 10).

Objet : **l'export ZIP de souveraineté** (`crates/kesh-api/src/exports/global.rs`) passe de 19 à
30 tables CSV ; chaque table hors export porte une justification écrite (`TABLES_HORS_EXPORT`) ;
trois gardes tiennent la partition contre `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs`) ;
`csv_sanitize` est appliquée aux cellules de texte ; `audit_log` est lue par une fonction dédiée,
non bornée, scopée par `company_id`.

## Lentille 1 — Blind Hunter (diff SEUL)

Tu ne lis **que** le fichier de diff. Revue adversariale générale : logique, erreurs, multi-tenant
(`company_id` sur **chaque** requête neuve, **y compris** les tables filles sans `company_id`
propre — lignes, items), injection de formule CSV (quelles cellules de texte libre échappent à
`csv_sanitize` ?), cohérence en-têtes / colonnes de chaque sérialiseur, mémoire (lectures non
bornées), tests qui ne prouvent rien ou passent à vide, incohérences code / commentaires / textes.

## Lentille 2 — Edge Case Hunter (diff + dépôt)

Chaque branche du code neuf, en lisant l'appelé et l'appelant. Priorités :
1. **Scoping** des dix lectures neuves : une table fille (`credit_note_lines`,
   `supplier_invoice_lines`, `payment_batch_items`) est-elle scopée par **jointure** sur son parent,
   et la jointure est-elle juste ? Un test multi-tenant couvre-t-il **chaque** table, ou quelques-unes ?
2. **Cohérence de l'instantané** : l'export lit trente tables par des requêtes successives sur le
   pool — une écriture concurrente peut-elle produire un export incohérent (ligne fille sans
   parent, total qui ne concorde pas) ? L'était-ce déjà avant ? Dire si la story l'aggrave.
3. **`csv_sanitize`** : relire la fonction ; chaque colonne texte des 30 sérialiseurs l'emploie-t-elle
   (description, nom, IBAN, référence, `details` JSON d'`audit_log`, libellés) ? Les montants et dates
   sont-ils laissés intacts ? Une valeur négative (`-12.50`) est-elle mutilée ?
4. **Les trois gardes** : prouvent-elles réellement la partition, ou sont-elles vraies par
   construction (même source des deux côtés) ? Une table ajoutée demain à `TABLES_TO_TRUNCATE` sans
   entrer dans l'un des registres fait-elle rougir un test **qui tourne en CI** (pas un
   `debug_assert` dans du code qui exige une base) ?
5. **Colonnes** : pour chaque table ajoutée, comparer les colonnes exportées au schéma réel
   (`crates/kesh-db/migrations/`, ou `crates/kesh-db/test-schema/`) — une colonne comptable
   oubliée (p. ex. `reverses_entry_id`, `project_id`, `settlement_*`, `company_id` d'`audit_log`) ?
6. **L'écran** (`frontend/src/routes/(app)/export/+page.svelte`) et les textes.

## Lentille 3 — Acceptance Auditor (diff + fiche + documentation)

Chaque AC (1 à 11) contre le diff. Les deux arbitrages et l'écart déclarés au Dev Agent Record —
justifiés ? écrits partout où il le faut ? Les décomptes **recomptés** depuis la source
(`TABLES_EXPORTEES` 30, `TABLES_HORS_EXPORT` 9, « trente cellules » sanitisées, « 10 lectures »,
« 12 lignes ajoutées » au CHANGELOG, tests E2E). ⚠️ **Le Dev Agent Record ne déclare AUCUN gate
exécuté** (ni backend, ni frontend, ni E2E) alors que l'AC 11 l'exige : le constater, dire ce qui
manque. Les textes : quatre locales, replis **mot pour mot** le FTL fr-CH. Le manuel
`docs/manual/fr/user-manual.tex` **et** `admin-manual.tex`, et leurs **PDF aplatis**
(`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, vers le scratchpad) :
⛔ **cette story ferme #386 — reste-t-il dans le dépôt une phrase qui dit que l'export omet les
achats, les avoirs, les projets ou la piste d'audit ?** Greper le manuel **entier**, le README,
`website/`, `CLAUDE.md`, les FTL. `docs/api-external.md` si l'export y figure. `CHANGELOG.md`
(section `[0.12.1]` seulement).

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve** (code lu,
  commande et résultat), correction. Pour tout CRITICAL ou HIGH affirmant qu'une chose est absente
  ou présente : la commande `grep -nF` exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `pdftotext` (vers le scratchpad) et
`cargo check` sont autorisés.
