# Prompt — revue de code P1, Story 30-2a (documents sans QR, « Documents à traiter », aperçu)

*Versionné le 2026-10-15. Trois lentilles (B, E, A), contexte frais chacune, lecture seule. Modèle : **Sonnet**
(passe complète — jamais Haiku pour une passe complète, D6 de la rétrospective de l'Epic 25).*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2`. Langue du rapport : français.

**Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-30-2 diff 5b566378 09c0057b -- . ':(exclude)_bmad-output'` —
`5b566378` = fin de la validation de la fiche (aucun code) ; `09c0057b` = **dernier commit de code**. Les commits qui
le suivent ne portent que la documentation (manuels et leurs PDF, CHANGELOG, README, `docs/api-external.md`,
`.env.example`), la fiche, le registre, `sprint-status.yaml` et ce prompt : la documentation se lit **au HEAD**
(`git diff 5b566378 HEAD -- docs CHANGELOG.md README.md .env.example`). ⚠️ Pathspec `':(exclude)…'`, **non** `':!…'`.
La branche **n'est pas encore rebasée** sur le merge de la 30-1a (développée en parallèle, C-30-2-27).

Fiche : `_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md` — **partie a seulement** :
AC1 à AC9, AC6-bis, AC9-bis ; tâches T-a0 à T-a9 ; § *Tests*, *Gardes existantes*, *E2E de parcours* (E1-a, E2-a,
E3), *Mutations* (M-a1 à M-a15, sans M-a13), *Angles morts assumés*, *Pièges connus* ; et le **Dev Agent Record**
(tests rouges, mutations, gates, relevé AC6-bis, écarts, conflits probables). La 30-2b et la 30-2c **ne sont pas**
l'objet de la revue : un manque qui leur revient n'est pas un finding (vérifie-le à la fiche avant de l'écrire).
Choix : `epic-30-choix-autonomes-302.md`, C-30-2-1 à 23 (fiche) et **C-30-2-24 à 29** (développement). Règles :
`CLAUDE.md` du dépôt — § *Migration breaking policy* **P1 à P8 en entier**, § *Recompter ses propres comptes rendus*,
§ *Propagation post-patch*, § *Le prompt d'une passe doit NOMMER le manuel*, § *Une passe qui ne déclare pas ses axes
exercés ne compte pas*, § *Un appariement automatique propose, il ne crée jamais*. Issue : #685 — **aucun mot-clé de
fermeture** dans un message de commit, même en prose : `git -C /home/gcorbaz/devel/kesh-30-2 log 5b566378..HEAD
--format=%B | grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?):?\s+#[0-9]+"` doit être vide.

⛔ **Lecture seule stricte.** N'écris, ne modifie, ne crée aucun fichier hormis ton rapport. **Aucun** `git checkout`,
`git switch`, `git stash`, `git reset`, `git commit`, `git tag`, `git push` — sous aucune forme. **Aucune commande qui
écrit, compile ou exécute** : ni `cargo`, ni `npm`/`npx`, ni `make`/`latexmk`, ni `scripts/prepare-release.sh`, ni
`scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `python3`, ni `node`, ni `sed -i`, ni `perl`. Aucune commande
`gh` qui écrit (`gh issue view` est permis). Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`,
`cat`, `ls`, `jq`, et `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` **vers la sortie standard uniquement** — jamais
vers un fichier.

⛔ **Données réelles — dépôt public.** N'ouvre rien hors du worktree et de `/home/gcorbaz/devel/kesh-gate-logs/`. Les
captures `kesh-gate-logs/302a-ac6bis/*.png` sont des fixtures synthétiques : tu peux les ouvrir.

## Axes communs aux trois lentilles (chacune les déclare exercés ou non)

1. **La migration** `crates/kesh-db/migrations/20261015000001_imported_documents_without_qr.sql` contre la politique
   P1–P8 : breaking déclarée, `UPDATE _kesh_version … '0.14.0'` en **dernière** instruction ; les **dix** crates à
   0.14.0 (`grep -n '^version' crates/*/Cargo.toml`, `Cargo.lock`) ; un seul `ALTER`, l'unicité **avant** le
   `DROP INDEX` (erreur 1553) ; la contrainte « tout ou rien » porte-t-elle **les sept** colonnes, et sa forme relue
   (`test-schema/0001_schema_squash.sql`, où MariaDB l'écrit sans parenthèses autour de `x is null`) garde-t-elle la
   même sémantique ? P5 : ligne d'audit et compteurs (`ls crates/kesh-db/migrations/*.sql | wc -l` ; `grep -c '^| \`20'
   docs/migrations-idempotence-audit.md` ; partition `yes`/`tracked-by-sqlx`/`no` recomptée) ; P6 : chaque site
   `migrations.len()` / `apply_migrations_up_to` (`grep -rn`) — `migrations_upgrade_path.rs` 82/48, frontière 34 ;
   P7 : `EXEMPT_MIGRATIONS` (19, justification `Durable`) ; P8 : checksum inscrit (`crates/kesh-db/migrations.sha384`)
   égal au fichier ; squash régénéré (`test_schema_guard`). Les tests de version (`downgrade_protection_*`,
   `migrations_kesh_version_initial_row`) disent-ils 0.14.0 ? Une base peuplée passe-t-elle (test
   `imported_documents_without_qr_preserves_existing_rows`, monté **par version**, assertion de montage) ?
2. **L'import** (`crates/kesh-api/src/inbox_import.rs`) : les quatre issues du décodage (`Ok(Some)` contrôlé,
   `Ok(None)`, `ImageDecode` refusé, autres erreurs `unreadable`) ; le contrôle pur **avant** l'archivage
   (`check_scanned_fits_columns`, `kesh-db/src/entities/imported_supplier_invoice.rs`) — longueurs en caractères, les
   onze colonnes contraintes, plage et décimales du montant, montant vide admis ; `from_scanned` / `without_qr` ; les
   deux filets (`DataLengthOrRange`, `CheckConstraintViolation`) et leur sort du fichier archivé ; le rapport
   (`qr`, `qrError` absent sauf `unreadable`, réactivation lue sur la ligne stockée) ; la trace de création (même
   forme, clés présentes à `null`, `qr_present`, `iban_present`) ; la constante `ERR_NO_QR_CODE_FOUND` disparue. Un
   chemin peut-il encore sortir en `500` ou en `400` global pour un seul fichier ? Un QR qui passe le contrôle peut-il
   violer une contrainte en base ?
3. **La complétion et le retrait** (`routes/imported_supplier_invoices.rs`) : sans QR, ni devise, ni routage, ni
   montant ; avec QR, **exactement** les contrôles d'avant (routage QR-IBAN ⇔ QRR) ; `iban_present` lu sur l'`Option` ;
   message de `CURRENCY_NOT_SUPPORTED`. Peut-on, par un état de base cohérent avec la contrainte, atteindre une
   complétion fausse (QR-IBAN sans QRR accepté, IBAN perdu) ?
4. **L'aperçu et la CSP** : `DocumentPreview.svelte` (lecture par `apiClient.getBlob`, types `image/png`,
   `image/jpeg`, `application/pdf`, tout autre type → lien seul ; 410 ; erreur ; révocation à la destruction **et** au
   changement de `load` ; course : une lecture lente d'un document précédent peut-elle afficher le mauvais ou fuir une
   URL ?) ; `previewLoader` de la page (fonction stable par document : un rendu relance-t-il la lecture ?) ;
   `middleware/csp.rs` — `img-src`, `frame-src`, `object-src` gagnent `blob:`, **et rien d'autre ne change**
   (`default-src`, `script-src`, `frame-ancestors`). Le relevé AC6-bis du Dev Agent Record tient-il aux captures et au
   journal `kesh-gate-logs/302a-ac6bis.log` ?
5. **L'écran « Documents à traiter »** (`routes/(app)/supplier-invoices/import/+page.svelte`) : titre, badge, date
   d'import (`createdAt` naïf, dix premiers caractères — angle mort UTC ?), « Retirer de la liste » et sa confirmation,
   rapport (`inbox-import-accepted-row`, « sans QR-code » / « QR illisible » + libellé de `qrError` + aide), menu,
   K5 (`FiscalYearChecks.svelte`, `closing-checks-labels.ts`), fiche de la facture (`[id]/+page.svelte`, « Voir le
   document »). Une ligne sans QR fait-elle encore lire un champ QR comme une chaîne (`row.creditorName` affiché
   « null » ? formulaire de complétion ?).
6. **i18n et gardes** : clés neuves et changées **×4 locales** (parité : mêmes clés dans les quatre fichiers), vraies
   traductions, mêmes variables, replis égaux à fr-CH au caractère près ; `-error-no-qr-code-found` retirée partout,
   `-error-qr-amount-out-of-range` ajoutée à `IMPORT_ERROR_LABELS` **et** à `MOTIFS_DYNAMIQUES` ; `sitesTotal`
   2193 → 2205 et `CLES_RELEVEES` 291 → 302 **recomptés depuis la source** aux deux bornes ; libellés d'audit changés
   (C-30-2-26). L'inventaire de vocabulaire d'AC8 (`grep -rnE "compléter|Compléter|Importer des factures|[Éé]cart|justificatif|facture d.origine|facture\(s\) importée|complét|importée"`
   sur les quatre locales, `frontend/src`, `docs/manual/fr`, `README.md`) : chaque site restant est-il légitime ou
   écrit comme conservé ?
7. **Les tests prouvent-ils ce qu'ils nomment.** Les tests neufs de `inbox_import_e2e.rs` (helper
   `qr_invoice_png_with`, substitution de lignes après `build_payload`, UTF-8 avec ECI), du dépôt (sept `INSERT` à une
   seule colonne `NULL`, unicité), de l'entité (onze colonnes à la limite et une au-delà), de `csp.rs`, de
   `csv_tables.rs`, Vitest (`DocumentPreview.test.ts`, `import-page.test.ts`, `FiscalYearChecks.test.ts`), E2E
   (`supplier-documents.spec.ts`, `inbox-import.spec.ts`, `helpers/inbox.ts` : poignée de main, copies uniques, **aucun
   `test.skip`**). La campagne M-a1 à M-a15 (journal `kesh-gate-logs/302a-mutations.log`, motifs dans le Dev Agent
   Record) : chaque « tuée » l'est-elle **pour la bonne raison** (assertion, non compilation) ? Cherche la **mutation
   plausible qui resterait verte** (exemples : `without_qr` qui laisserait `currency` à `"CHF"` ; `qr_present` calculé
   `true` ; réactivation qui dirait toujours `present` ; `previewLoader` recréé à chaque appel ; le rapport qui
   n'afficherait pas `qrError`). Un test qui passerait à vide ?
8. ⛔ **LA DOCUMENTATION — manuels et PDF, `api-external.md`, CHANGELOG, README.** Chaque phrase neuve ou modifiée
   dit-elle vrai **au code** ? Manuel utilisateur (`docs/manual/fr/user-manual.tex`) : § *Documents à traiter*
   (`sec:import-factures`) — tout document entre, le fichier quitte le dossier dès l'import, badge, trois gestes, ce qui
   reste refusé et `failed/`, CHF, renvoi à la sauvegarde de `KESH_DOCUMENTS_DIR` et à #503 ; K5 (§ *Préparer la
   clôture*). La phrase gardée « L'import ne se déclenche pas tout seul » est-elle **mot pour mot** (G5,
   `textes_coherents.rs`) ? Manuel d'administration (`docs/manual/fr/admin-manual.tex`) : § import
   (`sec:inbox-import`), lignes de `KESH_DOCUMENTS_DIR` (tableau, sauvegarde quotidienne, Synology), et
   **§ *Passer à la 0.14.0*** (`sec:maj-0-14-0`) — sauvegarde avant (dump, `.keshbackup` **et** documents), refus de
   démarrage d'une 0.13.x (`crates/kesh-db/src/version.rs`, `check_downgrade_protection`), retour par restauration,
   paragraphe *Intégrations par l'API*. `docs/api-external.md` : section *Documents à traiter — l'import du dossier*
   (codes, statuts HTTP, champs nullables — vérifie chacun contre `errors.rs` et les routes) ; K5. CHANGELOG
   `## [0.14.0] — Non publié` : bandeau *À faire*, *Ajouté*, *Modifié* ; `[0.13.4]` **intact**. README l. 44 (« import
   lancé depuis l'écran » gardé, G5). `.env.example`. Contrôle les **PDF aplatis** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` et de même `admin-manual.pdf`, **vers la
   sortie standard** — ⚠️ ligatures (« ﬀ », « ﬁ ») et apostrophe typographique (« ’ ») rendent un `grep` naïf faux
   négatif. Les PDF sont-ils postérieurs à la dernière modification de leur `.tex` (`git log -1 --format=%H -- <f>`) ?
   Cherche une phrase des manuels, du README, de `website/` ou de `docs/` que la story rend **fausse** et n'a pas
   touchée (`grep -rn "QR-facture\|porteu\|à compléter\|justificatif" docs/manual/fr README.md website docs/*.md`).
   Ce qui revient à la 30-2b / 30-2c n'y est **pas** annoncé (C-30-2-29) — est-ce bien le cas ?

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Les chemins d'erreur de l'import
  (archivage puis échec d'insertion, doublon en course, fichier disparu), la contrainte « tout ou rien » face à chaque
  écrivain de la table (`create_in_tx`, `mark_completed`, `mark_discarded`, `reactivate_to_complete`, restauration de
  sauvegarde — une sauvegarde 0.13.x restaurée en 0.14.0 ?), Svelte 5 (`$effect` qui écrit `view` sans le lire ;
  closure de `load` ; `{#each}` keyé), sécurité de l'aperçu (un document HTML ou SVG renommé `.png` : que fait le
  serveur, que fait le composant ? une `<iframe>` de même origine sur un `blob:` PDF peut-elle exécuter du script ?).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Exactement 70 / 71 caractères
  (accentués, combinants, emoji hors BMP — `chars()` compte des scalaires, MariaDB `utf8mb4` des caractères : égaux ?) ;
  montants `0.01`, `999999999.99`, `1000000000`, `-0`, `20.000`, `20.005`, exposant ; pays et devise vides ; QRR et
  SCOR de 40 / 41 caractères ; un PDF de plus de `KESH_INBOX_MAX_PDF_PAGES` pages ; un PNG suffixé ; un document
  réactivé dont la ligne est `completed` ; la date d'import près de minuit ; rejoue mentalement la campagne de
  mutations (axe 7).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (AC1 à AC9, AC6-bis, AC9-bis) contre le
  code : satisfait, partiel, absent ; T-a0 à T-a9 ; M-a1 à M-a15 présentes et conformes ; les écarts à la fiche
  sont-ils **tous** déclarés au Dev Agent Record (C-30-2-24 à 29, T-a0 différé, ce que la 30-2a n'écrit pas
  d'AC9-bis) ; les chiffres de gates et d'E2E déclarés correspondent-ils aux journaux `kesh-gate-logs/302a-*.log` ;
  les échecs E2E jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* ; `sprint-status.yaml` :
  clé `30-2a-documents-sans-qr` en `review`, commentaire daté ; **recompte depuis la source** chaque décompte du Dev
  Agent Record (tests neufs par fichier aux deux bornes `5b566378` / `09c0057b`, clés, mutations, gates) et vérifie
  son **périmètre**.

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/302a-review-p1-<B|E|A>.md` (ta lettre), et rien d'autre nulle
part :

- findings numérotés `B1…` / `E1…` / `A1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`,
  **preuve** (sortie `grep -nF` ou `sed -n` copiée pour toute affirmation de présence ou d'absence), correctif
  proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** (axes 1 à 8 et ceux de ta lentille) — un
  « 0 finding » sans cette liste ne compte pas comme passe ;
- un décompte final `CRITICAL/HIGH/MEDIUM/LOW`.
