# Prompt — revue de code P1 de la story v0200-corrections-mineures (#1348, #1336, #1342, #1349, #1343, #1375), trois lentilles Opus, première passe complète

Tu relis la story **v0200-corrections-mineures** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5 runes), release
**0.20.0**, Epic 48. Kesh tient les livres réels du Project Lead, qui a **déjà émis de vraies factures remises sur papier** : rien de
ce qui existe (facture, avoir, lignes, totaux, TVA, écritures, numéros, PDF) ne doit changer. **Aucune migration.** Six corrections :

- **#1348** — vingt et une clés appelées par le serveur (`crates/kesh-api/src/errors.rs`) n'existaient dans aucun catalogue : le
  serveur rendait la **clé brute**. Elles entrent aux quatre catalogues (`crates/kesh-i18n/locales/*/messages.ftl`, texte fr-CH =
  repli du code) et à la `TABLE` de `les_replis_rust_suivent_le_catalogue` (`crates/kesh-api/tests/textes_coherents.rs`) ; le refus
  du quota d'envoi (`BatchExceedsSendQuota`) passe de `t` à `t_args` (`{ $max }`) ; test écrit en dur
  `cles_de_1348_traduites_dans_les_quatre_locales` (`crates/kesh-i18n/src/loader.rs`). La garde générale est **hors périmètre**
  (#1406, milestone v0.21.0, décision de Guy du 2026-11-19).
- **#1336** — l'e-mail et le téléphone de la page des contacts perdent leur `maxlength` (coupe silencieuse d'un texte collé) ; le
  serveur refuse déjà au-delà de 320 et 50.
- **#1342** — un pays saisi seul, autre que « CH », est refusé au serveur (`address_input.rs::validate_optional_keeping`) et à l'écran
  (`countryAlone`, clé `contact-error-address-country-alone`) ; « CH » seul reste « pas d'adresse ».
- **#1349** — le motif d'un bouton d'enregistrement grisé s'affiche au-dessus des boutons (`contact-form-blocked-reason`), dérivé de
  `formValidation`, sauf les deux motifs déjà affichés sous leur champ.
- **#1343** — un prénom, un nom ou une raison sociale faits **uniquement** de caractères invisibles (U+00AD seul…) sont refusés en
  nommant le champ et le geste (`helpers.rs::optional_visible`, appelée par `onboarding.rs`, `contacts.rs::validate_common`,
  `contact_persons.rs`) ; l'écran des personnes de contact affiche désormais le motif du serveur (`ContactPersonsManager.svelte`).
- **#1375** — doc-comment de `isSettlementOutcomeUnknown` (`frontend/src/lib/shared/utils/unknown-outcome.ts`), écrit d'après la
  **fiche** de la 48-12b (cas ii de D6 : la 48-12b n'est pas fusionnée).

Worktree `/home/gcorbaz/devel/kesh-v0200-cm`, branche `story/v0200-corrections-mineures`, partie de `epic/48-corrections-0-20-0` =
`a051e17e5`. Diff à relire, **aplati** : `git -C /home/gcorbaz/devel/kesh-v0200-cm diff a051e17e5 d0efdb33e` (`d0efdb33e` = dernier
commit de code ; trois commits de code : `7eec3962e` #1348, `cca96e590` serveur #1343 / #1342, `d0efdb33e` écrans, manuel,
doc-comment, E2E). Les commits suivants ne portent que la fiche, le `sprint-status.yaml` et ce prompt — vérifie-le par
`git -C /home/gcorbaz/devel/kesh-v0200-cm diff --stat d0efdb33e HEAD`. Langue du rapport : français. **Modèle : Opus**, contexte frais,
trois lentilles en parallèle.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/v0200-corrections-mineures.md` en entier (« Découpage », « Ce qui se lit au code », D1 à D7,
AC1 à AC16, tableau des mutations, Note de version, **Dev Agent Record**, **Change Log**) et le dernier rapport de validation
`/home/gcorbaz/devel/kesh-gate-logs/v0200cm/validation-p4.md`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le
avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/` : `etape0.log`, les rouges
(`rouge-21-cles.log`, `rouge-quota.log`, `rouge-serveur.log`, `rouge-vitest.log`), les verts (`vert-1348.log`,
`gate-cible-serveur.log`, `vert-vitest.log`, `vert-vitest-2.log`), `mut/` (scripts `run-mut-rust.sh`, `run-mut-front.sh` ; un journal
par mutation `M1.log` … `M21.log` ; les rejeux E2E `e2e-M5.log`, `e2e-M9.log`
(+ `.build`, `.e2e`, `.e2e.backend`) et le script `run-mut-e2e.sh`), `gate-ci.log` et `gate-ci.console`, `front-*.log`,
`e2e-complet.log` (+ `.backend`), `manuel-avant.txt`, `manuel-apres.txt`, `captures.log`, `captures-10.log`, et les scripts
`backend-v0200cm.sh`, `captures-v0200cm.mjs`. `scripts/test-fast.sh --ci` **n'a pas tourné** (déclaré au Dev Agent Record).
**Captures** (thème clair, base de test jetable `kesh_e2e_v0200cm`, données factices) : `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/captures/`
— `01-entreprise-vide-motif.png` à `12-contact-fr-adresse-videe-motif.png` (liste au Dev Agent Record). Regarde-les si ton outil lit
les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part (ni
  scratchpad, ni `/tmp`) ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`/`merge`/`rebase`/`push`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `run-mut-rust.sh`, `run-mut-front.sh`,
  `backend-v0200cm.sh`, `captures-v0200cm.mjs` : ils se **lisent**) ; ni `make`, `latexmk`, `pdflatex`, `cargo`, `docker`, `mariadb`,
  `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **Les AC.** Chaque AC (AC1 à AC16) et chaque décision (D1 à D7) contre le code, les tests et les journaux. Un AC sans test qui peut
   rougir doit être déclaré filet, avec sa raison. Les écarts à la fiche sont-ils déclarés au Dev Agent Record ?
2. **Le manuel** (`docs/manual/fr/user-manual.tex`) : items « Nom », « Adresse structurée », « Email, Téléphone », le paragraphe neuf
   après la liste des champs du contact, l'encart des personnes de contact, et l'étape « Coordonnées légales » de l'assistant. Disent-
   ils ce que fait le code (320 et 50, refusés et non coupés ; pays seul refusé sauf CH ; vider le pays pour retirer l'adresse d'un
   contact étranger ; nom invisible refusé, le message nomme le champ et le geste ; motif près du bouton, ou sous le champ pour la
   raison sociale > 255 et le délai hors bornes ; alerte en tête de l'encart) ? Compare `manuel-avant.txt` et
   `sed -n '219p;1350p;1353p;1354p;1355,1362p;1394,1402p' docs/manual/fr/user-manual.tex` (recalcule les numéros par `grep -n`). Les
   manuels DE/IT/EN n'existent pas (`ls docs/manual/{de,it,en}`). Le **PDF n'est pas régénéré dans la branche** (il le sera à la
   release) : forme du contrôle, `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'un pays saisi seul'`
   — `0` attendu dans la branche ; le dire, ne pas le compter comme défaut. Même forme pour `'aucun caractère visible'` et
   `'la ligne au-dessus de lui'`. L'item « Compte comptable par défaut » (#1405) n'est pas touché : hors périmètre.
3. **Les mutations.** M1, M5 à M19, M21 (les numéros M2 à M4d et M20 sont sortis vers #1406) : chacune est-elle tuée **pour la bonne
   raison**, par le test que la fiche nomme, avec le décompte de rouges prévu (M8 : trois Vitest ; M9 : huit ; M10 : deux ; M1 et M19 :
   parité **verte**) ? Chaque journal montre-t-il le diff de la mutation, le rouge et sa raison, puis « == restauré » ? Les rejeux E2E
   de M5 et M9 rougissent-ils pour la raison de l'AC5 et de l'AC9 ? Un test vert par construction (mock qui rend l'attendu,
   `fireEvent.submit` qui contourne un bouton grisé, comparaison de vide à vide, liste comparée à elle-même) ?
4. **Cohérence écran / serveur.** La règle du pays seul est-elle la même aux deux bouts (rognage, casse : `to_ascii_uppercase` contre
   la normalisation de `countryAlone`) ? Un pays comme « ch » ou «  CH  » ? Un pays « Ch » mêlé de caractères non ASCII ? Le contrôle
   de l'écran (`trim()`) laisse-t-il partir un nom invisible que le serveur refuse — le refus s'affiche-t-il alors (`formError` sur la
   page des contacts, toast de l'assistant, alerte des interlocuteurs) ? `blockedReason` peut-il afficher un texte qui n'est pas celui
   qui grise le bouton, ou rester vide quand le bouton est grisé (hors des deux motifs sous le champ et de `formSubmitting`) ? Le
   panneau du fournisseur neuf (`SupplierCreatePanel.svelte`, non modifié) : un pays seul autre que CH y arrive-t-il, et son refus
   s'affiche-t-il ?
5. **Les contacts réels existants.** Un contact ou une société déjà enregistrés peuvent-ils être refusés à leur prochain enregistrement
   ? Lesquels exactement (nom entièrement invisible ; pays seul en base, état qu'aucune route n'écrit) — et est-ce écrit comme limite
   assumée (AC12 b, Dev Notes « Production ») ? Un contact sans adresse, un contact étranger complet, une société configurée se
   réenregistrent-ils ? Une route qui **recopie** l'état (PUT d'un contact tel quel, routes de `companies.rs`, restauration d'une
   sauvegarde, import d'installation, serveur MCP) passe-t-elle par `validate_common` ou `optional_visible` et peut-elle désormais
   refuser une valeur déjà en base ? Aucune facture touchée :
   `git -C /home/gcorbaz/devel/kesh-v0200-cm diff a051e17e5...d0efdb33e --stat -- crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report 'frontend/src/routes/(app)/invoices' frontend/src/lib/components/invoices`
   doit être vide ; dans `errors.rs`, seul le site du quota change.
6. **Les 21 clés aux quatre locales.** Chaque clé est-elle aux quatre catalogues, même ordre, texte fr-CH **égal** au repli de son site
   (continuations `\` comprises), traduction juste selon `docs/i18n-glossaire.md` (« lot » = Stapel / lotto / batch ; « exercice » =
   Geschäftsjahr / esercizio contabile / fiscal year ; « rappel » = Mahnung / sollecito / reminder ; « justificatif » = Beleg /
   documento giustificativo / supporting document ; « journal d'audit » = Audit-Protokoll / registro di audit / audit log ;
   « réconciliation » = Abstimmung, manuelle Abgleich) ? `{ $max }` présent aux quatre ? La liste du test de `loader.rs` égale-t-elle
   le tableau de la fiche (recopiée, non extraite) ? La clé d'écran `contact-error-address-country-alone` : quatre locales, juste après
   `contact-error-address-npa-city`. `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts`) : 3502 → 3503 — un site neuf, juste ?
   `dette-parite-connue.txt` inchangé ?
7. **La fiche et la Note de version.** Le Dev Agent Record dit-il vrai (journaux et commits cités ; décomptes recomptés depuis le diff
   : tests Rust ajoutés, `it(` / `test(` ajoutés ; rouges E2E jugés fichier par fichier) ? La Note de version dit-elle vrai, issue par
   issue ? Le doc-comment de #1375 dit-il ce que la fiche de la 48-12b (branche `story/48-12-reponses-perdues-fournisseurs`, lisible
   par `git show story/48-12-reponses-perdues-fournisseurs:_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`)
   décide (E6, E8, #986 hors périmètre) ?

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, d'état, de sécurité, de régression ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et états limites, axes 3 à 5 surtout : valeurs vides, blancs, invisibles mêlés,
  casses du pays, bascule de type Entreprise / Personne, contact rouvert, conflit de version, refus du serveur resté dans `formError`
  après correction, motifs successifs.
- **A — Acceptance Auditor** : chaque AC et chaque décision contre le code, les tests, les journaux et les captures (axes 1, 2, 6, 7).

## Rapport

Un rapport par lentille : `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/review-p1-B.md`, `…/review-p1-E.md`, `…/review-p1-A.md` (le
seul fichier que tu écris). Pour chaque finding : identifiant (`B1`, `E1`, `A1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et
ligne, extrait cité, scénario, correctif proposé. Verdict chiffré en tête. **Liste des axes exercés ET non exercés** (un « 0 finding »
sans elle ne compte pas). Ce qui n'a pas tourné reste écrit : la revue n'exécute rien ; les gates qui ont tourné au passage en revue
sont cités au Dev Agent Record avec leurs journaux.
