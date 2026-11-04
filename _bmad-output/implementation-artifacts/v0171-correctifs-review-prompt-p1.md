# Story v0171-correctifs — revue de code P1 (trois lentilles B, E, A — modèle Sonnet)

⛔ **Lecture seule stricte.** Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni `make`,
ni `latexmk`, ni `python3`, ni `node`, ni `docker`, ni `curl`, ni `claude`, ni aucun script du dépôt
(`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`… compris), ni `sed -i`, ni `rm`, ni
`pkill`, ni `killall` ; **aucune redirection** (`>`, `>>`, `tee`), aucun fichier temporaire, aucun fichier dans le
scratchpad ; aucun `git checkout`, `switch`, `reset`, `stash`, `rebase` sous aucune forme. Permis : `git log`, `git show`,
`git diff`, `git grep`, `/usr/bin/grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext f.pdf -` vers la sortie standard
seulement. Le **seul** fichier que tu écris est ton rapport. Un fichier créé par erreur se déclare au rapport, il ne se
supprime pas. ⛔ **Aucune pièce réelle** : ni `~/kesh/`, ni `~/kesh-test/`, ni le dossier de projet de Guy, ni le NAS —
les fixtures de la story sont synthétiques et suffisent.

⚠️ **Dans ce shell, `grep` est une fonction vers `ugrep`** : pour toute citation ou tout contrôle, utilise
`/usr/bin/grep` ou `git grep`.

Langue : français soigné. Contexte frais : tu n'as lu aucune passe précédente.

## Ce que tu relis

- Worktree : `/home/gcorbaz/devel/kesh-0171`, branche `release/0.17.1`, partie de `910e5d5a` (merge de la v0.17.0).
- Le diff du développement : `git -C /home/gcorbaz/devel/kesh-0171 diff 68374eb9..HEAD` (code, tests, manuel, PDF,
  CHANGELOG, fiche) ; les commits `910e5d5a..68374eb9` ne portent que la fiche et ses validations.
- La fiche : `_bmad-output/implementation-artifacts/v0171-correctifs.md` (AC-1 à AC-12, T1 à T14, M1 à M7, Dev Agent
  Record) ; le registre : `v0171-choix-autonomes.md` (C-v0171-1 à 22).
- Issues : #854 (P2, QR-facture à lignes CR+LF), #866 (quantité brute), #871 (dates ISO fournisseur et lots), #878
  (champs quantité et prix d'un brouillon de facture cliente), #879 (dates ISO des écrans clients). Hors périmètre, déjà
  tracées : #880, #881, #882.
- Les journaux du développement : `/home/gcorbaz/devel/kesh-gate-logs/v0171/` (rouges, mutations, gates, recette).
- Les **captures** : `/home/gcorbaz/devel/kesh-gate-logs/v0171/captures/` (dix-sept PNG, thème clair, et
  `recette.log`) — ouvre-les.

## Les trois lentilles

Une lentille par agent ; chacune écrit son rapport :

- **B (Blind Hunter)** — le diff seul, sans la fiche : défauts de code, de tests et de textes visibles au diff.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0171-review-p1-B.md`.
- **E (Edge Case Hunter)** — chaque branche et chaque borne : `est_payload_spc` (LF, CR+LF, `\r` seul, en-têtes mêlés,
  `SPC\n0200` sans suite, chaîne vide, BOM), le parseur sur CR+LF (dernière ligne, lignes 32 à 34, `\r` interne) ;
  `readableAmount` sur les valeurs du serveur (`0.0000`, négatif, illisible, `1E+2`) et aux trois sites d'`InvoiceForm` ;
  `formatSwissDate` sur `''`, `null` contourné par `?? ''`, un horodatage sans `slice`, une échéance nulle.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0171-review-p1-E.md`.
- **A (Acceptance Auditor)** — chaque AC contre le code livré et le Dev Agent Record : ce qui est déclaré a-t-il tourné,
  les décomptes se recomptent-ils, les écarts à la fiche sont-ils écrits ?
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0171-review-p1-A.md`.

## Axes obligatoires (chaque lentille les exerce ou dit pourquoi non)

1. **#854, la règle unique** : `SPC_HEADERS` / `est_payload_spc` employés par `first_spc` et par le test de
   `decode_spc_from_png_roundtrip` ; aucune autre définition (`git grep -nE 'SPC.?n0200|starts_with\("SPC'`) ; les
   négatifs ; le parseur inchangé ; Kesh émet toujours LF (`generator.rs` non touché).
2. **#854, les tests** : T1, T2, T4, T8 rouges avant correctif (journal `rouges-backend.log`), T2 établit que `rxing`
   rend le `\r` ; T5–T7 ; M1–M3 et leurs tests (`mut/back.log`) — M3 laisse vert le jumeau inexploitable CR+LF de T4 :
   l'écart est-il juste et écrit ?
3. **Données existantes** : aucune migration, aucune écriture au démarrage, rien de relu d'office ; le retour à la
   0.17.0 sans restauration est-il vrai ?
4. **#866 / #878** : `readableAmount` réutilisé (aucun helper neuf, C-v0171-6), importé depuis la feature fournisseur
   dans des écrans clients ; l'aller-retour (ce qui part au `PUT`) ; `vatRate` laissé tel quel (C-v0171-21, #882) ;
   l'E2E `invoices.spec.ts` passé de `150.0000` à `150.00`.
5. **#871 / #879 : inventaire des sites** — rejoue le critère d'AC-8 sur `frontend/src --include=*.svelte` et celui
   d'AC-12 sur **les huit fichiers énumérés au § 10 de la fiche**, avec le second filtre `| /usr/bin/grep -vF
   formatSwissDate` ; compare à la liste nominative du Dev Agent Record. Cherche une date ISO restée brute sur ces
   écrans, hors du motif (autre nom de champ, ligne enveloppée).
6. **Les tests Vitest** : un par site, `not.toContain` de l'ISO ; un test qui passerait à vide (testid absent, portail
   non lu, valeurs identiques avant/après) ; les mutations M4–M7 (`mut/front.log`, `mut/front-liste.log`) — chacune
   rougit-elle le seul test de son site ?
7. **AC-9 (#871)** : le texte de la confirmation capturé par Playwright sur la base jetable (`captures/recette.log`) ;
   la conclusion « déjà corrigée en 0.17.0 » est-elle établie ?
8. **Le manuel — `.tex` ET PDF** : `docs/manual/fr/user-manual.tex`, § *Documents à traiter* (la phrase sur la 0.17.1
   et le CR+LF) et *Faire relire le QR d'un document* ; chaque phrase vraie au code. Lis le PDF aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` ; pour compter,
   `timeout 60 sh -c "pdftotext f.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oF 'motif' | wc -l"` (apostrophes
   typographiques `’` dans le PDF). Aucun autre écran ne décrit-il une date ISO ou une quantité `1.0000` ?
9. **`CHANGELOG.md`** : section `## [0.17.1] — Non publié`, cinq entrées *Corrigé*, la promesse « retour à la 0.17.0
   sans restauration » ; versions Cargo, macros `kesh-style.sty`, `release.yml` **non** touchés.
10. **Les commits** : `git log 910e5d5a..HEAD --format=%B | /usr/bin/grep -niE
    '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit être vide.

## Le rapport

Pour chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), ligne lue copiée, défaut,
remède proposé. Toute affirmation de présence ou d'absence s'appuie sur une sortie `/usr/bin/grep -nF` ou `git grep`
copiée. Termine par :

- la liste des **axes exercés** et celle des **axes non exercés**, avec la raison ;
- le décompte final : CRITICAL n · HIGH n · MEDIUM n · LOW n.

Un « 0 finding » sans la liste des axes exercés ne compte pas comme passe.
