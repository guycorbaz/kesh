# Prompt — revue de code P1 de la story `v0190-nom-societe` (#1097), trois lentilles, première passe complète

Tu relis la story **`v0190-nom-societe`** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5 runes), release
**0.19.0** (ou 0.20.0 si elle glisse) : **le nom de la société devient modifiable depuis *Paramètres → Organisation*.** Une route
dédiée `PUT /api/v1/companies/current/name` (Admin, session seulement), qui juge le nom par les règles de la QR-facture (vide au
sens de `champ_vide_pour_qr`, 70 caractères, jeu de caractères) **avant** de lire la société, verrou optimiste, no-op sans
version ni audit, `company.updated` borné au nom ; un bloc d'édition du nom sur le patron du bloc e-mail (`syncOpenBlocks`,
conflit 34-6) ; les trois refus du PDF qui portaient « ne se modifie pas encore depuis l'interface » renvoient à l'écran (trois
replis Rust, douze phrases FTL), deux gardes inversées ; quatre clés i18n ; deux phrases du manuel utilisateur ; quatre compteurs
de routes et le compte du manuel d'administration recomptés (C-v0190c-23). **Aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-v0190n`, branche `story/v0190-nom-societe`. Diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-v0190n diff b2e2ff67c e5b1f0161` (`b2e2ff67c` = pointe de `epic/v0190-correction`, fiche
validée ; `e5b1f0161` = dernier commit de code ; les commits suivants ne portent que la fiche, le registre, ce prompt et
`sprint-status.yaml` — vérifie-le par `git -C /home/gcorbaz/devel/kesh-v0190n diff --stat e5b1f0161 HEAD`). Langue du rapport :
français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/v0190-nom-societe.md` en entier (« Ce qui se lit au code », D1 à D7, AC1 a à h et g-bis,
AC2 à AC7, plan de coupure, **Change Log**, **Dev Agent Record**, Note de version) ; le registre
`_bmad-output/implementation-artifacts/e0190-choix-autonomes-v0190c.md` (C-v0190c-4 à 10, 14, 15, 18, **23, 24**) ; les rapports
de validation `/home/gcorbaz/devel/kesh-gate-logs/v0190c/validation-nom-societe-p3-ciblee.md` et `validation-p4-ciblee.md`.
Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/v0190n/` : `cible-1.log` (le premier rouge des compteurs), `cible-2.log`,
`vitest-settings-1.log`, `vitest-i18n-1.log` (le rouge de `sitesTotal`), `vitest-i18n-2.log`, `check-1.log`, `lint-i18n-1.log`,
`e2e-company-name-2.log` (+ `.backend.log`, `.captures.log`), `mut/` (les scripts `mutations-rust.py`, `mutations-front.py` et
un journal par mutation), `test-fast-review.log`, `gate-ci-review.log` (rouge KF-061, #1126) et `gate-ci-review-2.log` (vert), `front-*-review.log`. **Captures** (thème clair, base
jetable, voie 2) : `/home/gcorbaz/devel/kesh-gate-logs/v0190n/captures/` (`1-organisation-bouton-modifier.png`,
`2-edition-du-nom.png`, `3-refus-caractere-hors-jeu.png`, `4-refus-du-pdf-renvoie-a-organisation.png`) et leur script
`captures.mjs`, lancé par `e2e-et-captures.sh` — regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (ils se **lisent**) ; ni `make`, `latexmk`, `cargo`,
  `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **La route dit vrai à la QR-facture.** `validate_company_name_for_qr` (`crates/kesh-api/src/routes/companies.rs`) applique-t-elle
   exactement ce que le PDF appliquera (`kesh-qrbill/src/validation.rs`, `validate_address` : vide, longueur, jeu, dans cet ordre) au
   nom **tel qu'il sera rendu** ? Un nom accepté par la route peut-il encore faire refuser le PDF (rognage, U+00AD au milieu, U+2028,
   `\r`, caractères combinants, un nom que `texte_code` normalise) ? Un nom refusé par la route serait-il accepté par le PDF ? Le
   compte en caractères (`chars().count()`) est-il celui de `check_len` ?
2. **Ordre, verrou, audit, full-replace.** Le nom se juge avant la lecture de la société : un nom invalide à version périmée est un
   `400`. `CompanyUpdate` reporte-t-il **tous** les autres champs (`first_name`, `last_name`, adresse structurée, IDE, `org_type`,
   langues, e-mail, téléphone, site) ? Le cas « société d'avant les composants » (g-bis) ? Le no-op (nom identique après rognage) ne
   bumpe ni n'audite ? `details_json` = `{before:{name}, after:{name}}` sans autre clé, acteur `user` ? La route est-elle dans le bloc
   `admin_routes` (`require_not_pat`, `crates/kesh-api/src/lib.rs`) ? Un Comptable, une Consultation, une clé d'API reçoivent `403` ?
3. **Les registres et compteurs.** `admin_pat_denied_e2e.rs` (liste et répartition `(15, 39)`), `audit_route_registry.rs`
   (inventaire 147, tracées 140, `SansEcritureAuJournal` 120, registre 150, en-tête du module), `audit_acteur_garde.rs` (deux
   lignes, `lib.rs:478`) : chaque nombre se recompte-t-il depuis la source, et les commentaires historiques ne sont-ils pas
   réécrits ? Le manuel d'administration (« 140 des 147 routes à verbe mutant », « 140 + 4 + 3 = 147 ») : le changement est-il
   légitime au regard de C-v0190c-15 (rien dans la note de montée de la 0.17.4) ? Un autre site (README, `website/`, `docs/`) dit-il
   un compte de routes devenu faux (`git grep -nE "\b(139|146)\b"` trié à la main) ?
4. **Les refus du PDF** : trois replis (`invoice_pdf_service.rs`) et douze phrases (quatre `messages.ftl`), tournures et accords
   (« -le » pour le nom ; de-CH « ihn », it-CH « -lo ») ; la garde inversée (`texte.contains("Organi")`) et `invoice_pdf_e2e.rs`
   (`paramètres → organisation` en minuscules) ; `cles.len() == 42` inchangé. Reste-t-il une phrase « ne se modifie pas encore »
   (et ses équivalents de/it/en) quelque part : code, FTL, frontend, manuels, `website/`, README ?
5. **L'écran** (`frontend/src/routes/(app)/settings/+page.svelte`) : bloc nom transposé du bloc e-mail — `startNameEdit`
   (préremplissage du nom stocké tel quel, même refusé), `saveName` (refus local du vide, envoi rogné, 400 → message du serveur et
   saisie gardée, 409 → `reloadAfterConflict` avec `NAME_FIELDS`, `nameValuesOf`), `syncOpenBlocks` (branche `nameEditing`),
   `adoptCompany` ; `maxlength={70}` ; bouton réservé à `isAdmin` ; `col-span-2` pendant l'édition. Une séquence (deux blocs
   ouverts, relecture plus ancienne, échec de relecture, Annuler puis Modifier) donne-t-elle un 409 parasite, un avis périmé, une
   saisie perdue, ou un nom affiché qui n'est pas celui enregistré ? Captures.
6. **Les tests et les mutations** : chaque cas d'AC1 (a à h, g-bis), d'AC2, d'AC4 et d'AC5 a-t-il un test qui peut rougir ? Les
   mutations jouées (`mut/`) et celles déclarées équivalentes tiennent-elles ? Un test vert par construction (montage vide,
   comparaison de vide à vide, mock qui renvoie ce qu'on attend) ? `the_name_is_trimmed_line_breaks_at_the_edge_included` exerce-t-il
   vraiment U+2028 en bord ? L'E2E (`frontend/tests/e2e/company-name.spec.ts`) rend-il le nom tel qu'il l'a trouvé, et sa
   restauration est-elle sûre si le test échoue en route ?
7. **Le manuel utilisateur** (`docs/manual/fr/user-manual.tex`, § *Vos coordonnées sur la facture* et § *Les caractères d'une
   QR-facture*) : dit-il ce que fait le code (70 caractères, jeu de la QR-facture, refus en français, personne physique, PDF figé,
   effets sur e-mails, avoirs, rapports, ordres de paiement) ? `git grep -n -E "1097|modifie pas encore" -- docs/manual/fr/` est-il
   vide ? « depuis la version~0.19.0 » est à reposer à l'intégration si la story glisse (fiche, D7 (2)) : ne pas le compter comme
   défaut. Le **PDF n'est pas régénéré dans la branche** : forme du contrôle à la release,
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'fait exception'` — dans la branche il
   rend encore `1`, c'est attendu ; le dire, ne pas le compter comme défaut. Même chose pour
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '139 des 146'`.
8. **i18n** : quatre clés × quatre catalogues, rangées après `settings-company-email-conflict-reload-failed`, mêmes textes que les
   clés e-mail pour le conflit ; `sitesTotal` 3087 → 3095 (`frontend/src/lib/shared/i18n-keys.test.ts`), huit littéraux, se
   recompte-t-il depuis le diff ?
9. **La fiche et le registre** : le Change Log cite-t-il des lignes réellement lues ? Le Dev Agent Record dit-il vrai (décomptes de
   tests recomptés : `git grep -c` des `#[sqlx::test]`, `#[test]` et `it(` ajoutés, journaux cités, commits) ? La Note de version
   dit-elle vrai ? C-v0190c-23 et 24 sont-ils cohérents avec le code ?

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, de sécurité, d'état ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et séquences (axes 1, 2 et 5 surtout), états limites (nom stocké invalide,
  société sans composants, concurrence, échec de relecture, rôles, clé d'API).
- **A — Acceptance Auditor** : chaque AC et chaque décision de la fiche contre le code, les tests et les journaux (axes 3, 4, 6 à 9).

## Rapport

Un rapport par lentille : `/home/gcorbaz/devel/kesh-gate-logs/v0190n/review-p1-B.md`, `…-E.md`, `…-A.md`. Pour chaque finding :
identifiant (`B1`, `E1`, `A1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario, correctif
proposé. Verdict chiffré en tête. **Liste des axes exercés ET non exercés** (un « 0 finding » sans elle ne compte pas). Ce qui n'a
pas tourné reste écrit : la revue n'exécute rien ; `test-fast --ci`, `gate-ci` et le frontend complet ont tourné au passage en
revue (journaux cités au Dev Agent Record), l'E2E complet (et le `gate-ci` de clôture) non — seul `company-name.spec.ts` a tourné.
