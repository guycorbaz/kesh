# Prompt — revue de code P1 de la fiche `v0174-correctifs-1` (#819, l'import CAMT.053), trois lentilles, Sonnet

Tu relis le développement de la fiche **`v0174-correctifs-1`** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit,
MariaDB 10.11), release **0.17.4**, **sans migration** : *l'import CAMT.053 lit en entier les noms, libellés et
références qui portent une référence d'entité (`&amp;`, `&#xE9;`) ou une section CDATA (#819), sans qu'un relevé ou une
ligne déjà importés par la 0.17.3 cessent d'être reconnus comme doublons.* Trois lentilles, **une par rapport** :
**B** (chasseur aveugle : le diff seul, sans la fiche), **E** (cas limites : chaque branche, chaque borne), **A**
(auditeur d'acceptation : chaque AC, chaque mutation, chaque déclaration du Dev Agent Record contre les pièces).

Worktree `/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs`. Le diff de cette fiche, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0174c show 16039590` (le commit de la fiche 1) ; contexte : `git diff 3bb418b7 16039590`.
Langue du rapport : français. Modèle : **Sonnet**. Passe **complète** (première passe). Contexte frais.

Rapports (un fichier par lentille, **le seul fichier que tu écris**) :
`/home/gcorbaz/devel/kesh-gate-logs/v0174c-review-p1-1-B.md`, `…-1-E.md`, `…-1-A.md`.

## À lire (lentilles E et A ; la lentille B lit le diff d'abord, la fiche ensuite seulement pour classer)

- La fiche `_bmad-output/implementation-artifacts/v0174-correctifs-1.md` (AC-1.1 à AC-1.7, *Tests et mutations*,
  MA-819-1 à 7, Dev Notes, **Dev Agent Record**, Change Log) ; l'index `v0174-correctifs.md` (§ *Manuel*, § *CHANGELOG*) ;
  le registre `v0174-choix-autonomes-v0174c.md`, **C-v0174c-3 à 7, 21, 22, 23**.
- Le Dev Agent Record déclare des rouges, des mutations et des gates : **ne les prends pas pour argent comptant**,
  recoupe-les (lecture seule) avec `/home/gcorbaz/devel/kesh-gate-logs/v0174c/rouges.log`, `rouges-819-import.log`,
  `rouges-819-core.log`, `cible-819-api.log`, `mut/mutations.log`, `mut/MA-819-*.log`, `nextest-final.log`,
  `gate-final.log`, `e2e-*.log`.
- Code : `crates/kesh-import/src/camt053/{mod,v04,v08}.rs`, `tests/camt053_{tests,golden}.rs`,
  `tests/fixtures/camt053_golden/v04_characterization.0.17.4.json`, `tests/fixtures/README.md` ;
  `crates/kesh-core/src/bank_imports.rs` (`detect_duplicate_lines_with_aliases`) ;
  `crates/kesh-api/src/routes/bank_imports.rs` (`legacy_0173_aliases`, aperçu, création, CSV), `tests/bank_imports_e2e.rs`
  (`*_819`). Source de quick-xml : `~/.cargo/registry/src/*/quick-xml-0.42.0/` (lecture).
- Manuel : `docs/manual/fr/user-manual.tex` (§ *Import dans Kesh*, § *Détection des doublons*, § *Création d'une règle*)
  **et** son PDF `docs/manual/fr/user-manual.pdf`, aplati.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier hormis ton rapport. **Aucun fichier temporaire** (ni
  `/tmp`, ni scratchpad, ni `kesh-gate-logs`), **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`).
- **Aucun `rm`** sous aucune forme ; **aucun `git` qui écrit** (`checkout`, `switch`, `restore`, `commit`, `stash`,
  `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Rien qui compile ou exécute** : ni `cargo`, ni `npm`/`npx`/`node`, ni `python`, ni `perl`, ni `make`/`latexmk`,
  ni `docker`, ni `mariadb`/`mysql`, ni `sqlx`, ni `curl`, ni `claude`, ni `sed -i`, ni **`scripts/prepare-release.sh`**,
  ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun script de `kesh-gate-logs` (`mutate.sh`, `vmutate.sh`,
  `lourd.sh`, `gate-final.sh`…). **Aucune commande `gh`.** Aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`,
  `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, la lecture d'images,
  `timeout 60 pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` puis `grep -cF`/`grep -oF` à **motifs fixes** (jamais
  `grep -E` à quantificateur sur le PDF aplati).
- Toute affirmation de présence ou d'absence s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée au rapport.
- **Données réelles** : aucune (XML, IBAN, noms inventés). Ne lis rien sous `~/kesh/`, `~/kesh-test/`.

## Axes obligatoires, pour chaque lentille

1. **Lecture identique à la 0.17.3 en mode hérité.** `parse_legacy_0173` (configuration du lecteur, distribution
   morceau par morceau, `GeneralRef` et `CData` ignorés) rend-elle, pour tout document, exactement ce que rendait la
   boucle de `7bd4d5c0` ? Relis `git show 7bd4d5c0:crates/kesh-import/src/camt053/mod.rs`. La caractérisation héritée
   égale-t-elle **toutes** les références ?
2. **Lecture corrigée.** Le tampon est-il vidé avant **tout** autre événement (Start, End, Empty, Comment, PI, Decl,
   DocType, Eof), avec le chemin encore celui de l'élément ? Blancs intérieurs gardés, extérieurs rognés ; CRLF du
   CDATA ; entité inconnue → `MalformedXml` au message de Kesh (inventaire `KESH_MESSAGES`) ; référence de caractère
   invalide ; `resolve_xml_entity` (C-v0174c-22). La lecture corrigée change-t-elle autre chose que les trois valeurs
   déclarées de `v04_characterization` ?
3. **L'alias de clé.** `legacy_0173_aliases` : jamais d'erreur ; alias seulement s'il diffère ; alignement par index ;
   les **deux** sites CAMT (aperçu, création) et **aucun** site CSV ; `detect_duplicate_lines_with_aliases` ne compte pas
   deux fois. Un **faux doublon** possible (une ligne neuve dont la clé héritée égale celle d'une ligne existante
   différente) : est-il borné comme la fiche le dit, et signalé à l'aperçu ?
4. **Les tests prouvent-ils ce qu'ils disent ?** AC-1.5 (montage « 0.17.3 » par le dépôt, (b), (c), (d)), AC-1.6
   (deux tests nommés du repli), AC-1.7 (texte tronqué relu). Chaque mutation MA-819-1 à 7 est-elle **tuée** par le
   test que la fiche nomme (journaux `mut/MA-819-*.log`) ? Une mutation manquante ou survivante ?
5. **Le manuel et son PDF** contre le code : la phrase d'entité inconnue (message cité **tel que l'écran le montre**,
   `crates/kesh-i18n/locales/fr-CH/messages.ftl`), les doublons, la règle *Contrepartie exacte* ; le CHANGELOG `[0.17.4]`.
   Le PDF aplati porte-t-il chaque phrase neuve ?
6. **`kesh-import` reste publiable** : `#[doc(hidden)]`, aucune API publique cassée (`v04::parse`, `v08::parse`
   gardent leur signature), `cargo publish --dry-run` au journal `publish-final.log`.

## Rapport

Pour chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier:ligne, preuve (`grep -nF` / `sed -n` copiés),
remède proposé. Puis la **liste des axes exercés** et **celle des axes non exercés** (avec la raison). Un « 0 finding »
sans cette liste ne compte pas comme passe. Décompte final recompté depuis la liste.
