# Prompt — revue de code P1 de la Story 34-11a1 (compte sans IBAN, #972), trois lentilles, première passe complète

Tu relis la **Story 34-11a1** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.6**,
Epic 34 : **un compte sans IBAN** — la **seule migration de la release, breaking** (`20261110000001_bank_accounts_without_iban` :
`iban` nullable, `account_kind`, `card_last4`, quatre `CHECK`, `kesh_version_min_required` à 0.17.6, dix crates en 0.17.6), l'entité
(`iban: Option<String>`, `BankAccountKind`, `card_last4`), les refus nommés (`409 PAYMENT_BATCH_ACCOUNT_HAS_NO_IBAN` à la création et à la
génération d'un lot, `422 BANK_IMPORT_CAMT_NEEDS_IBAN` à l'aperçu et à l'import CAMT), l'empreinte d'aperçu à IBAN `null`, l'audit
`iban_present` lu, l'export `bank_accounts.csv`, la QR-facture sans panique, et la tolérance de l'écran (`format.ts`, tableau, accueil,
*Paramètres*, sélecteurs, écrans des lots et de l'import). Aucune route ne crée encore de compte sans IBAN (34-11a2).

Worktree `/home/gcorbaz/devel/kesh-e34a1`, branche `story/34-11a1-compte-sans-iban`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e34a1 diff 07f714a2 93e2642f` (`07f714a2` = base ; `93e2642f` = dernier commit de code ; le compte rendu
suit). Langue du rapport : français. Modèle : **Sonnet ou Opus** (première passe : jamais Haiku). Contexte frais.

⚠️ **Les propriétés centrales.**
1. **Les refus se fondent sur `iban IS NULL`, jamais sur le type** (C-e341-18).
2. **Un compte bancaire garde exactement l'empreinte d'aperçu de la 0.17.5** (valeur figée), et un IBAN absent ne se confond avec aucune chaîne.
3. **Aucun site qui lit l'IBAN d'un compte bancaire ne plante sur `None`** : ni `unwrap`, ni `expect`, ni décodage `(String,)` résiduel.
4. **La migration ne touche aucune ligne existante**, et une base 0.17.5 peuplée monte sans écart.
5. **Le retour à la 0.17.5 passe uniquement par la sauvegarde d'avant la mise à jour** — et le manuel le dit en tête.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche `_bmad-output/implementation-artifacts/34-11a1-compte-sans-iban.md`
en entier (inventaire S1-S18 et C1-C6, AC-1 à AC-13, T0 à T6, mutations M1 à M8, **Change Log** et **Dev Agent Record**) ; les registres
`e34-choix-autonomes-e341.md` (C-e341-13, 14, 17, 18, 19, 21, 29, 39) et `e34-choix-autonomes-e34a1.md` (C-e34a1-1 à 10). Ne prends pas le
Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e34a1/` :
`rouges.log`, `t1-inventaire-avant.txt`, `t1-inventaire-apres.txt`, `p6-grep.txt`, `cible-1.log`, `gate-test-fast-1.log`, `build-1.log`,
`gate-ci-1.log`, `gate-ci-journal-1.log`, `clippy-1.log`, `clippy-2.log`, `cible-2-admin-import.log`, `front-check-1.log`, `front-check-2.log`,
`front-lint-i18n-1.log`, `front-unit-1.log`, `front-unit-2.log`, `front-build-1.log`, `e2e-r1/playwright.log`, `mut/campagne.log` et `mut/M*.log`
(le script `mut-e34a1.py` et `mut-campagne-e34a1.sh` sont à **lire**, jamais à exécuter), `recette-montee-1.log`, `recette-montee-journal/`
(dont `apres.log`, `image-refus.log`, `binaire-montee.log`), le crochet `apres-sans-iban.sh`, `recette-complement-e34a1.sh` et
`recette-c/resume.txt`, `manuels-*.log`, `flat-admin-manual.txt`. **Captures** (thème clair) sous
`/home/gcorbaz/devel/kesh-gate-logs/e34a1/captures/` : `34-11a1-01-tableau-comptes.png`, `34-11a1-02-accueil.png`,
`34-11a1-03-parametres.png`, `34-11a1-04-import-carte.png`, `34-11a1-05-lot-sans-la-carte.png` — regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`,
  `scripts/regen-test-schema.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`,
  `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **La migration et P1 à P8 du `CLAUDE.md`** (axe obligatoire) : P1/P3 — `MODIFY COLUMN` ⇒ bump présent, **dernière instruction** ; P2-bis —
   les dix `crates/*/Cargo.toml` à 0.17.6 dans le **même commit** que la migration (`git show --stat 194faa78`) ; P4 — aucun
   `breaking-skip-bump` ; P5 — la ligne d'audit, et les compteurs **recomptés** (`ls crates/kesh-db/migrations/*.sql | wc -l`,
   `/usr/bin/grep -c '^| \`20' docs/migrations-idempotence-audit.md`, partitions `yes`/`tracked-by-sqlx`/`no` recomptées au tableau, en-tête et
   ligne `Total`) ; P6 — rejoue `git grep -n "migrations.len()\|apply_migrations_up_to" crates/` et juge chaque site (la frontière de
   `upgrade_path_preserves_data` est-elle restée à 34 ? `\b(56|57|90|91)\b` grepés dans `migrations_upgrade_path.rs`) ; P7 — l'entrée
   `EXEMPT_MIGRATIONS`, sa justification (une sauvegarde 0.17.5 importée prend `account_kind` à son `DEFAULT` : **vrai** ? `is_required()`
   d'une colonne `NOT NULL DEFAULT` ; `card_last4` nullable), le compteur 23 → 24 ; P8 — la migration et le registre
   `crates/kesh-db/migrations.sha384`. Le squash régénéré est-il exactement le schéma migré (les quatre `CHECK`, l'ordre des colonnes) ?
   `chk_bank_accounts_iban_nonempty` gardée : le raisonnement « un `CHECK` à `NULL` est satisfait » est-il juste en MariaDB 10.11, et testé ?
   Les `CHECK` couvrent-ils tous les invariants de C-e341-17, et rien de plus (par exemple : une carte **close** ou **archivée**) ?
2. **Le tri des assertions `"0.17.0"`** (AC-2) : chaque site gardé ou porté à `"0.17.6"` l'est-il à juste titre ? `git grep -n "0\.17\.0\"" -- 'crates/'`
   et `git grep -nE "\b0\.17\.6\b" -- crates/ docs/ CHANGELOG.md` : un résidu qui désigne le minimum courant est-il resté à 0.17.0 ?
3. **L'inventaire clos des sites (S1-S18)** : rejoue la commande de T1 (fiche) et compare aux deux fichiers `t1-inventaire-*.txt`. Chaque site A
   refuse-t-il **sur `iban.is_none()`**, au bon rang (lot : après archivé et clos ; CAMT : **après** la détection du format, **avant** l'empreinte
   et la lecture) ? Cherche un lecteur oublié : `git grep -nE "\.iban\b" -- 'crates/*/src/**'` hors `creditor_iban`, `counterparty_iban`,
   `qr_iban`, `account_iban` ; côté écran, `npm run check` vert est-il le seul filet, et les sept sélecteurs reçoivent-ils bien des objets
   à `iban: string | null` ? Le `SettleInvoiceDialog` et `supplier-invoices/[id]` affichent-ils une carte sans planter ?
4. **Les manuels et la documentation contre le code** : `docs/manual/fr/admin-manual.tex` § *Passer à la 0.17.6* et le paragraphe *Compatibilité*,
   et le **PDF** aplati (`docs/manual/fr/admin-manual.pdf` : « Passer à la 0.17.6 », le nom de la migration entier, le message FATAL, « 409 ») ;
   `docs/api-external.md` (note ¹¹ du § 7, § *empreinte d'aperçu*, tableau du § 10) ; `CHANGELOG.md` § `[0.17.6] — Non publié` (avertissement
   de sauvegarde **en tête**). Chaque phrase est-elle vraie au code (par exemple : « `GET /payment-batches/{id}/pain001` rend le même refus » ;
   « la réponse des écritures porte `accountKind` » pour `PATCH`, `close`, `reopen`) ?
5. **Les témoins et les mutations** : chaque AC a-t-il un test qui peut rougir (`mut/M*.log` : M1 à M8 tuées, par le test attendu) ? L'écart déclaré
   « rouges d'abord » (`rouges.log`) est-il honnête ? Propose des mutations plausibles non jouées (par exemple : `camt_target_iban` retiré de
   `create_camt_under_lock` seul ; le refus du lot placé **après** le contrôle du compte lié ; `bankAccountFullId` qui rend l'IBAN court ;
   `camtAllowed` vrai pour `iban === undefined` ; le `CHECK` `chk_bank_accounts_card_last4` sans le terme `account_kind`).

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne la migration, l'entité et son `Decode`, `columns!()`, `is_no_op_change`, `create_batch` et `generate_pain001_xml`,
`camt_target_iban` et ses trois appels, `preview_fingerprint`, `full_update_audit_state`, `serialize_bank_accounts_csv`,
`BankAccountJson`, `invoice_pdf_service`, les deux mappages d'erreur, `format.ts`, `BankImportUpload.svelte` (`camtAllowed`, `accept`, la zone
de dépôt), l'écran des lots, les types. Cherche : un `Option` aplati en `""` ; un ordre de refus qui change un code d'erreur existant ; un `$derived`
qui lit un état non réactif ; un test qui passe **à vide** (fixture qui ne contient pas le cas, `find` qui rend `undefined`).

## Lentille E — Edge (chaque branche, chaque borne)

Carte **archivée**, **close**, liée ou non au grand livre ; `card_last4` `NULL` ; IBAN `""` ou fait d'espaces (refusé par la base ?) ; aperçu
**CSV** d'une carte puis import **CAMT** du même fichier renommé ; une carte choisie à l'import puis un compte avec IBAN (le champ `accept` suit-il ?) ;
un serveur 0.17.5 qui ne rend pas `accountKind` (le parseur du frontend) ; l'export d'une base mêlant comptes et carte ; un `.keshbackup` 0.17.5
importé dans une 0.17.6 (les colonnes neuves absentes du fichier) ; l'onboarding qui met à jour le principal (`is_no_op_change` avec un principal
sans IBAN est-il atteignable ?). Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC-1 à AC-13 : livré, testé, au lieu prescrit ? Les gates déclarés au Dev Agent Record ont-ils tourné, au commit dit (journaux) ?
Le dernier commit de code, `93e2642f`, n'a qu'un gate ciblé : l'écart est-il déclaré, et acceptable ? La recette de montée (cinq phases) et son
complément (refus 409, import du fichier `pre-upgrade/`, empreintes) répondent-ils à T5 point par point ? Les deux limites voisines demandées en
issue sont-elles réelles (capture 02) ? Le CHANGELOG, le manuel, `api-external.md` : conformes à l'AC-13 ? Le Dev Agent Record n'affirme-t-il que
ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e34a1-review-p1-B.md`, `/home/gcorbaz/devel/kesh-gate-logs/e34a1-review-p1-E.md`,
`/home/gcorbaz/devel/kesh-gate-logs/e34a1-review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH /
MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**.
