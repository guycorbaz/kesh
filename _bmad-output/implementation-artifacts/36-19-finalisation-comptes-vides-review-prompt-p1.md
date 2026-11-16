# Prompt — revue de code P1 de la Story 36-19 (refs #1200, refs #987), trois lentilles, première passe complète

Tu relis la **Story 36-19** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.18.0**
(sans migration), Epic 36, phase 2 : **la finalisation de l'assistant complète les comptes vides des réglages de
facturation**. Un `GET /api/v1/company/invoice-settings` fait avant la fin de l'assistant (journal, comptes bancaires,
factures) crée une ligne de réglages sans compte ; la finalisation la refusait ensuite (« 1100, 3000 manquants »). Elle la
complète désormais, sans remplacer un compte déjà choisi, et, si elle doit refuser, nomme le réglage, sa cause et son
remède. Un candidat lié à un compte bancaire ou non imputable n'est jamais désigné (partie « finalisation » de #987). Kesh
tient des livres réels (0.17.8, finalisée : la correction ne touche pas une installation finalisée).

Worktree `/home/gcorbaz/devel/kesh-36-19`, branche `story/36-19-finalisation-comptes-vides`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-19 diff 6c4f2229e 92266e805` (`6c4f2229e` = l'étape 0, fiche validée ; `92266e805` =
le dernier commit de code ; `git log --oneline 6c4f2229e..HEAD` donne la suite : les rouges `6fb456e85`, le squelette
`57948b58d` — qui porte aussi le manuel et `MULTI-TENANT-SCOPING-PATTERNS.md` —, le correctif `0723be024`, le test du
bras au catalogue chargé `92266e805`, puis la fiche, le registre, `sprint-status.yaml` et ce prompt). Langue du rapport : français.
Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-19-finalisation-comptes-vides.md` en entier (*Ce qui se lit au code*, D1 à D7,
AC1 à AC11, tests, mutations M1 à M14, **Change Log** et **Dev Agent Record**) ; le registre
`e36-choix-autonomes-36-19.md` (C-36-19-1 à -11) et `e36-choix-autonomes-e360.md` (C-e360-3, -11). Ne prends pas le Dev
Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/e36-19/` : `rouges.log`, `rouges-avant-correctif.log`, `rouges-squelette.log`,
`cible-apres-correctif.log`, `mut/*.log` (scripts `mut/muter.py`, `mut/jouer.sh` : à **lire**), `clippy-*.log`,
`test-fast-ci-*.log`, `gate-ci-*.log`, `e2e-*.log`, `front-*.log`. Déjà tracés, à ne pas rapporter : **#1201** (l'ordre
global des verrous de `docs/MULTI-TENANT-SCOPING-PATTERNS.md` contraire au code de la 15-6c), la partie « désignation de
rôle » de **#987** (`PUT /api/v1/accounts/{id}`), **#710** (refus de la route en français seulement).

**Livrables visibles** : aucun écran ne change (la Story 36-17, en parallèle, porte l'écran de fin ; n'en juge pas) ; ce
qui change est le **résultat de `POST /api/v1/onboarding/finalize`** (`200` au lieu de `400` sur une ligne vierge ; le
**message** du `400 VALIDATION_ERROR`) et ce que le journal d'audit inscrit (`company_invoice_settings.updated`,
`trigger` = `onboarding`). **Le manuel** : `docs/manual/fr/user-manual.tex`, ligne 217 (§ *Onboarding*) ; son PDF n'est
**pas** régénéré dans la story (à l'intégration, AC9) : contrôle le `.tex`, et dis si le PDF
`docs/manual/fr/user-manual.pdf` porte encore l'ancien paragraphe (attendu), lu **aplati**, sortie à l'écran :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` (forme E6, sans
redirection). `docs/MULTI-TENANT-SCOPING-PATTERNS.md:333` (ligne de `finalize`). La **note de version** de la fiche
(`## Note de version`) — la branche n'écrit pas `CHANGELOG.md` (C-e360-11). Le manuel dit-il **exactement** ce que le code
fait, cas par cas (ligne neuve, vierge, enregistrée ; obligatoire, facultatif) ? C'est en vérifiant le manuel qu'on trouve
les chemins oubliés. Pas de capture : aucun contrôle visible ne change.

⚠️ **Ce qui n'a pas tourné** : l'**E2E complet** et le second `gate-ci` — à la clôture (consignes de l'Epic 35) ; la spec
neuve `frontend/tests/e2e/onboarding-finalize-reglages.spec.ts` a été jouée **seule** (rouge contre le binaire d'avant le
correctif, vert après), journaux `e2e-*.log`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part.
- **Aucune redirection, sous aucune forme** : ni `>`, `>>`, `<`, `tee`, ni `2>`, `&>`, **ni `2>&1`**, **ni `2>/dev/null`**, ni
  `> /dev/null`. **Aucune substitution de processus** (`<(…)`, `>(…)`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`commit`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `mut/jouer.sh`, `mut/muter.py`, `cible.sh`,
  `bases-a-neuf.sh`, `e2e-spec.sh` : à **lire**, non à lancer) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`,
  `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`. `pdftotext` est permis, sortie à l'écran.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext … -`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`ls` ou `git cat-file -e <ref>:<chemin>`, E9).

## Les axes obligatoires, pour chaque lentille

1. **`complete_default_accounts_in_tx`** (`crates/kesh-db/src/repositories/company_invoice_settings.rs`) : les trois états
   de ligne (absente, vierge, enregistrée) ; `is_blank` (les **deux** moitiés, `version = 1` **et** dix comptes vides) ;
   `COALESCE` sur chaque colonne ; `version + 1` seulement sur une ligne existante ; aucun `UPDATE` sans colonne à poser ;
   les trois comptes de TVA jamais posés ; une ligne neuve égale à celle d'avant (colonnes de texte par leurs `DEFAULT`
   du schéma, non plus écrites par l'`INSERT` : vérifie au squash `crates/kesh-db/test-schema/0001_schema_squash.sql:341-346`).
2. **Les verrous** : ordre réel des lectures (rôles et plan `FOR UPDATE`, `INSERT IGNORE`, ligne `FOR UPDATE`, comptes
   désignés `LOCK IN SHARE MODE`, candidats relus `FOR UPDATE`, `bank_accounts` partagé) ; **V3-1** — la relecture de
   `postable` du candidat est-elle verrouillante, et T-D4s prouve-t-il ce qu'il dit (un cliché réellement pris avant le
   sous-compte, une lecture simple qui l'aurait vu imputable — cf. `mut/M14.log`) ? Le doc-comment de l'interblocage neuf
   est-il vrai (`routes/bank_accounts.rs`, `retry_app_on_deadlock`) ? Une lecture non verrouillante restée dans le chemin ?
3. **Le refus** (D5, D6) : chaque cause atteignable, l'ordre débiteurs puis produit, les deux examinés avant de refuser ;
   le message de `finalize_refusal_message` (`crates/kesh-api/src/routes/onboarding.rs`) **mot pour mot** contre le
   gabarit de la fiche (continuations `\` de Rust : une espace perdue ou doublée ?) ; `400 VALIDATION_ERROR` ; rien
   d'écrit sur refus (ligne neuve absente après `rollback`) ; l'enveloppe `insert_with_defaults_in_tx` ramène le refus à
   `InactiveOrInvalidAccounts` (démonstration, jeux de pièges, `seed_demo`, sa boucle de rejeu `kesh-seed/src/lib.rs`).
4. **Les facultatifs** : créanciers lié ou non imputable laissé vide sans refus ; sur une ligne enregistrée, rien de
   facultatif posé ; un réglage déjà désigné jamais recontrôlé ni remplacé.
5. **L'audit** : `company_invoice_settings.updated` (`trigger`, `before`, `after` au format de `settings_snapshot_json`,
   rendue `pub`) à la place de `…created` pour une ligne complétée, rien pour « inchangé », `…created` gardé pour une ligne
   neuve (séquence de `full_path_b_flow` inchangée) ; attribution `from_current_user` (clé d'API comprise ?).
6. **Le bras de rendu et la clé** (`crates/kesh-api/src/errors.rs`, `crates/kesh-db/src/errors.rs`,
   quatre `messages.ftl`, table G9 de `crates/kesh-api/tests/textes_coherents.rs`) : code, statut, repli égal au FTL,
   traductions justes (`de-CH`, `it-CH`, `en-CH`, libellés de menu des autres clés de chaque locale).
7. **Les tests** : rouges prouvés pour la **bonne** raison (`rouges.log`) ; un test qui passerait à vide (montage non
   vérifié, sonde qui ne prouve rien) ; les tests existants (`company_invoice_settings_repository.rs`,
   `onboarding_path_b_e2e.rs`, `onboarding_audit_e2e.rs`, `kesh-seed`) passent **sans modification** de leurs assertions
   (AC7 : `git diff 6c4f2229e HEAD -- <fichier>` ne touche que des ajouts en fin de fichier ?).
8. **Mutations** (`mut/*.log`) : chacune tuée par le test nommé à la fiche ; chaque journal finit par « restauré » ; les
   écarts déclarés (M11 joué en deux formes) ; propose des mutations plausibles non jouées.
9. **Manuel, note de version, `MULTI-TENANT-SCOPING-PATTERNS.md`, doc-comments** (`accounts.rs:405-428`,
   `onboarding.rs` autour de `refuse_future_claim_account` et de `finalize`) : chaque phrase dit-elle le code ?
10. **Le compte rendu** : chaque chiffre de gate cité avec son journal et son commit ; le Dev Agent Record n'affirme-t-il
    que ce qui a tourné ?

## Lentille B — Blind (le diff seul, sans la fiche)

Relis `company_invoice_settings.rs`, `onboarding.rs`, les deux `errors.rs`, puis les tests. Cherche : un compte écrasé ; un
chemin qui écrit sur refus ; un `version` faux ; une lecture sous cliché ; un commentaire devenu faux (« `INSERT IGNORE` …
idempotent », « 1100, 3000 ») ; un import devenu inutile.

## Lentille E — Edge (chaque branche, chaque borne)

Ligne absente, vierge, enregistrée aux dix comptes vides, enregistrée avec ses comptes ; débiteurs désigné archivé et produit
vide sans rôle (deux causes) ; compte désigné d'une autre société ; plan sans compte d'arrondi ; compte de rôle **et**
désigné identiques ; rôle porté par un compte archivé ; rejeu sur interblocage (la tentative annulée n'écrit rien) ;
finalisation à l'étape 8 (sortie avant toute écriture) ; un `PUT` concurrent ; un `PATCH` de liaison concurrent ; une clé
d'API. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC11 : livré, testé, au lieu prescrit ? Les LOW V3-1 à V3-4 traités à l'étape 0, ligne lue citée ? Les
choix C-36-19-9 à -11 exacts ? Les gates du passage en `review` : chaque chiffre au journal, au dernier commit de code ?
L'E2E déclaré « à la clôture » ? AC10 : `ls crates/kesh-db/migrations/*.sql | wc -l` = 91, cinq modules au plus.

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e36-19/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**.
