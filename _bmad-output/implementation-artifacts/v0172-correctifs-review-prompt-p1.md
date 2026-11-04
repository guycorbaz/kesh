# Story v0172-correctifs — revue de code P1 (trois lentilles B, E, A — modèle Opus)

⛔ **Lecture seule stricte.** Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni `make`,
ni `latexmk`, ni `python3`, ni `node`, ni `docker`, ni `curl`, ni `claude`, ni aucun script du dépôt
(`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`… compris), ni `sed -i`, ni `rm`, ni
`pkill`, ni `killall` ; **aucune redirection** (`>`, `>>`, `tee`), aucun fichier temporaire, aucun fichier dans le
scratchpad ; aucun `git checkout`, `switch`, `reset`, `stash`, `rebase` sous aucune forme. Permis : `git log`, `git show`,
`git diff`, `git grep`, `/usr/bin/grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext f.pdf -` vers la sortie standard
seulement. Le **seul** fichier que tu écris est ton rapport. Un fichier créé par erreur se déclare au rapport, il ne se
supprime pas. ⛔ **Aucune donnée réelle** : ni `~/kesh/`, ni le NAS, ni le dossier de projet de Guy — les fixtures sont
synthétiques.

⚠️ **Dans ce shell, `grep` est une fonction vers `ugrep`** : pour toute citation ou tout contrôle, utilise
`/usr/bin/grep` ou `git grep`.

Langue : français soigné. Contexte frais : tu n'as lu aucune passe précédente.

## Ce que tu relis

- Worktree : `/home/gcorbaz/devel/kesh-0172b`, branche `story/v0172-correctifs`. Le développement :
  `git -C /home/gcorbaz/devel/kesh-0172b diff 1be0f701..HEAD` (code, tests, i18n, manuel `.tex` et PDF, CHANGELOG,
  fiche, registre) ; `git log --oneline 1be0f701..HEAD` en donne les commits.
- La fiche : `_bmad-output/implementation-artifacts/v0172-correctifs.md` (sous-stories A2, B, C ; AC-A7 à AC-A10,
  AC-B1 à AC-B12, AC-C1 à AC-C4 ; mutations MA11, MA12, MB1 à MB10, MC1 à MC6 ; Dev Agent Record) ; le registre :
  `v0172-choix-autonomes.md`, section `v0172b` (C-v0172b-1 à 34 ; 30 à 34 nés au développement).
- Issues : #840 (P2, import simultané), #870 (liste des factures fournisseur), #851 (menu), #882 (taux de TVA
  désactivé), #880 (dates ISO), #883 (six LOW), #885 (date d'une facture créée depuis un document). **#876 n'est pas
  dans cette story** (sortie de la 0.17.2, CR #891).
- Les journaux : `/home/gcorbaz/devel/kesh-gate-logs/v0172b/` — `rouges.log` (rouges avant correctif, dont l'**oracle
  du symptôme de #851**), `mut/*.log` (une mutation par fichier), journaux de gate.
- Les **captures** : `/home/gcorbaz/devel/kesh-gate-logs/v0172b/captures/` (PNG, thème clair ; relevés JSON de
  l'oracle) — ouvre-les.

## Les trois lentilles

Une lentille par agent ; chacune écrit son rapport :

- **B (Blind Hunter)** — le diff seul, sans la fiche : défauts de code, de tests et de textes visibles au diff.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0172b-review-p1-B.md`.
- **E (Edge Case Hunter)** — chaque branche et chaque borne : `create_under_account_lock` (les quatre bras du `match`,
  un `RELEASE_LOCK` en échec après un commit, le refus « profil CSV introuvable » sous verrou, l'empreinte d'aperçu
  avant le verrou) ; la liste fournisseur (`?status=` inconnu, `?limit=0`, `?offset=-3`, `?offset` au-delà du total,
  `status` et `cancelled` ensemble, deux gestes rapprochés et `requestSeq`) ; `inactiveRates` et `_vatPlaceholder`
  (taux vide, lignes neuves après résolution, `addFreeLine` avant résolution) ; `dueFollows` (date vidée, fournisseur
  désélectionné, `defaultPaymentTermsDays` à 0) ; `formatSwissDate` / `formatSwissDateTime` (`Z`, `±HH:MM`, fraction,
  espace, heure d'été, `''`, chaîne non datée) ; `isExpired` (avec et sans `Z`).
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0172b-review-p1-E.md`.
- **A (Acceptance Auditor)** — chaque AC contre le code livré et le Dev Agent Record : ce qui est déclaré a-t-il
  tourné, les décomptes se recomptent-ils, les écarts à la fiche sont-ils écrits ?
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0172b-review-p1-A.md`.

## Axes obligatoires (chaque lentille les exerce ou dit pourquoi non)

1. **#840, l'ordre « commit puis rendu du verrou »** (angle mort déclaré, C-v0172b-28) : vérifie-le à la forme du code
   (`create_camt_under_lock`, `create_csv_under_lock` commitent avant de rendre ; la fermeture de `with_account_lock`) ;
   l'ordre des refus inchangé ; la saisie manuelle passée par le helper partagé (C-v0172b-30) sans changement de
   comportement ; TA5 à TA7 (`bank_imports_e2e.rs`, tests `v0172_*`) et MA11, MA12 (`mut/MA11.log`, `mut/MA12.log`).
2. **#870** : l'adresse, source de vérité lue par `$derived` ; le `page` réactif du test
   (`page-state.test-helper.svelte.ts`) ; MB1, MB2, MB8 ; l'E2E `supplier-invoices.spec.ts` (55 brouillons).
3. **#851** : l'oracle (`rouges.log`, relevés `captures/v0172b-851-avant-releves-*.json`) a-t-il rougi pour la bonne
   raison ? Le correctif (`aside` minoré, `<nav>` `max-h` + `overflow-y-auto`) ; AC-B5 bis, AC-B5 ter (C-v0172b-32),
   AC-B6 ; les captures avant/après.
4. **#882** : serveur inchangé (`git diff 1be0f701..HEAD -- crates/kesh-api/src/routes/vat.rs
   crates/kesh-api/src/routes/invoices.rs` vide) ; MB4, MB9 ; aucune marque d'interface envoyée au serveur.
5. **#885** : date **locale** (`todayLocal`), fuseau épinglé et horloge fixée dans les tests ; MB5, MB6 (le test a été
   corrigé après une première survie de MB6 : vérifie l'explication), MB7, la sonde MB10 ; les sites du § 22 mis à jour.
6. **#880, inventaire** : rejoue AC-C3 (motifs M₁, M₂ du § 23 de la fiche, sur les 32 fichiers) et compare aux 20
   lignes de la fiche ; le 25ᵉ site hors inventaire (C-v0172b-31) ; cherche une date ISO restée brute sur ces écrans,
   hors du motif. L'exception de l'expiration d'une clé (`formatExpiryDate`) et `isExpired` en UTC (MC5, MC6).
7. **#883** : .1 (reflux du doc-comment, aucune autre ligne), .3 (une ligne retirée), .4 (`v0172_est_payload_spc_bornes`),
   .5 (fixture CR+LF complète, `InvalidIban` attendu, MC3) ; .2 sans objet (aucun Svelte refluxé).
8. **i18n** : clés neuves dans les quatre `messages.ftl`, mêmes clés partout ; `fiscal-years-checks-k8-link` modifiée
   dans les quatre.
9. **Le manuel — `.tex` ET PDF** : `docs/manual/fr/user-manual.tex` (K8, la liste fournisseur, « Les dates », « Créer
   la facture d'un document », les lignes de facture) ; chaque phrase vraie au code. Lis le PDF aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (apostrophes `’` dans le PDF ; césures
   `réinitia- lisation`).
10. **`CHANGELOG.md`** : section `## [0.17.2] — Non publié` ; **aucune** puce sur #876 ; promesse « retour à la 0.17.1
    sans restauration » vraie (aucune migration : `ls crates/kesh-db/migrations/*.sql | wc -l` = 88) ; versions Cargo,
    `kesh-style.sty` non touchés.
11. **Les commits** : `git log 1be0f701..HEAD --format=%B | /usr/bin/grep -niE
    '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit être vide.

## Le rapport

Pour chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), ligne lue copiée, défaut,
remède proposé. Toute affirmation de présence ou d'absence s'appuie sur une sortie `/usr/bin/grep -nF` ou `git grep`
copiée. Termine par :

- la liste des **axes exercés** et celle des **axes non exercés**, avec la raison ;
- le décompte final : CRITICAL n · HIGH n · MEDIUM n · LOW n.

Un « 0 finding » sans la liste des axes exercés ne compte pas comme passe.
