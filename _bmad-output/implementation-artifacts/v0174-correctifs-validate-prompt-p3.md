# Prompt — validation P3 des fiches v0174-correctifs (release 0.17.4), Sonnet, passe ciblée sur la remédiation de la P2

Tu valides, en **passe ciblée** (`CLAUDE.md`, § *La passe ciblée*), la remédiation de la validation P2 des fiches de la story
`v0174-correctifs` de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), release corrective **0.17.4**, sans
migration. Worktree `/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs` ; le code est celui de `7bd4d5c0`
(seuls des fichiers `_bmad-output` ont changé depuis). Langue du rapport : **français**. Modèle : **Sonnet**. **Une seule
lentille**, **contexte frais** : ne te fie ni aux fiches ni aux rapports antérieurs ; relis au code.

**Ton périmètre est le seul commit de la remédiation**, `git show c3be330f` : ce qu'il ajoute, retire ou récrit dans
`_bmad-output/implementation-artifacts/v0174-correctifs.md` (index), `v0174-correctifs-1.md`, `-2.md`, `-3.md`,
`v0174-choix-autonomes-v0174c.md` et `sprint-status.yaml`. Le reste des fiches n'est relu que pour juger une ligne de ce commit
(contradiction, propagation manquante). Pour savoir ce qui était demandé, lis `/home/gcorbaz/devel/kesh-gate-logs/v0174c-validate-p2.md`
— pour le comprendre, non pour t'y fier.

## Axes

1. **Chaque ligne du diff est-elle vraie au code ?** En particulier :
   - `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`, `sitesNonResolus`, `:843-860`, `:1199`) : la prescription de
     recompte est-elle juste et complète ?
   - la garde de #889 **sans masque des commentaires** : rejoue toi-même
     `git grep -nE 'toISOString\(\)\.slice\(0, ?10\)' -- frontend/src ':!*.test.ts' ':!*.test-helper*'` et
     `git grep -nE 'formatSwissDate\(.*\.slice\(0, ?10\)' -- frontend/src ':!*.test.ts' ':!*.test-helper*'` ; un commentaire
     porte-t-il l'un des motifs ? Le retrait du masque crée-t-il un faux positif, ou laisse-t-il une mutation sans test ? Les
     « quatre fichiers » et les huit entrées sont-ils justes ?
   - les deux simulacres `ContactResponse` (`SupplierInvoiceForm.test.ts:38`, `supplier-proposals.test.ts:8`) : sont-ils les
     seuls (`git grep -n 'ContactResponse' -- frontend/src`) ?
   - fiche 1 : le chemin du mode jusqu'à `parse_with_version` (`crates/kesh-import/src/camt053/{mod.rs,v04.rs,v08.rs}`) ; les
     deux sites CAMT (`git grep -n 'parse_camt053(' -- crates/kesh-api/src`) ; le test (ii) — un IBAN écrit
     `CH93&#48;…` est-il bien lu tronqué par le mode hérité (`handle_stmt_text`, `= Some(txt)`), et la lecture corrigée le
     rend-elle entier ? `select_statement_by_iban` (`routes/bank_imports.rs:932-`) refuse-t-il alors comme la fiche le dit ?
     Le test (i) (date en CDATA) échoue-t-il dans le mode hérité à l'endroit dit ?
   - fiche 2 : AC-2.3 (`git grep -n PERIOD_LOCKED -- crates/kesh-api/tests`) ; les deux sites « fin de session » écrits
     autrement (`routes/bank_imports.rs:1478-1481`, `crates/kesh-db/src/backup.rs:616-621`) ; le DTO du MCP
     (`crates/kesh-api/src/mcp/outils/contacts.rs`).
   - index : la puce CHANGELOG #817 (« Pour l'API ») est-elle vraie au code prescrit (fiche 2, § 2.2) ? Le point de retour
     arrière à la 0.17.3 (#819) est-il juste (relis `normalize_reference_fallback`, `crates/kesh-core/src/bank_imports.rs`) ?
2. **Propagation** : chaque valeur corrigée par le commit a-t-elle été corrigée **partout** dans l'index, les trois fiches et
   le registre (hors entrées datées des Change Logs, qui ne se réécrivent pas) ? Greppe toi-même les valeurs anciennes :
   `masqu`, `MC-889-h`, `trois dans des fichiers`, `n'a pas de nombre en dur`, `MA-819-1 à 6`, `trois sites CAMT`,
   `phrase suivante`, `sans issue`, `:920-`, et les nouvelles pour leur cohérence (`2826`, `402`, `quatre`, `(10)`, `(11)`).
3. **Le commit introduit-il un défaut neuf** — une contradiction entre deux fiches, une AC devenue indécidable, une mutation
   sans test qui la tue, une prémisse fausse ?
4. **Le manuel** (`docs/manual/fr/user-manual.tex`, et le PDF aplati : `LC_ALL=C.UTF-8 pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) :
   les phrases du § *Manuel* de l'index touchées par le commit (#817 : « se modifie sans toucher au pays ») restent-elles
   vraies au code prescrit ? `sprint-status.yaml` : YAML lisible à l'œil (indentation, aucune clé dupliquée), clés
   `v0174-correctifs`, `-1`, `-2`, `-3` cohérentes avec l'index.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois** (outil Write), à `/home/gcorbaz/devel/kesh-gate-logs/v0174c-validate-p3.md`.
Il contient :

1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), la fiche, le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède. Une affirmation de présence
   ou d'absence sans sortie copiée ne compte pas.
2. **Les axes exercés** et **non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne
   compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture** hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`, `tee`),
  pas même dans le scratchpad. Un fichier créé par erreur se **déclare** ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`.
- Aucune commande `gh` qui écrit ; `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `mariadb`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt
  (les lire avec `cat` est permis).
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/Projets actuels/kesh/`, ni le NAS ;
  aucune donnée réelle dans le rapport.
- `grep` est une fonction vers `ugrep` dans ce shell : emploie `/usr/bin/grep` ou `git grep`.
- Commandes permises : `cat`, `sed -n`, `/usr/bin/grep`, `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `awk` (lecture),
  `git log`, `git show`, `git diff`, `pdftotext … -` vers la sortie standard, `gh api` en `GET`.
