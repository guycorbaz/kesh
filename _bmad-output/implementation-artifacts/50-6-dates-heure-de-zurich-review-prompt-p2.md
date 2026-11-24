# Prompt — revue de code P2 (passe ciblée) de la story 50-6 « Dates et heures de Zurich » (#1511)

Tu relis **un seul commit**, la remédiation de la revue P1 de la story **50-6** de Kesh (Rust/Axum, MariaDB, SvelteKit), Epic 50, release
0.20.0. Passe **ciblée**, **une seule lentille**, contexte frais, modèle **Sonnet** (la P1 était Opus). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-50-6`, branche `story/50-6-dates-heure-de-zurich`. Le commit à relire est le **dernier** de la branche
(`git log -1`, titre « remédiation de la revue P1 ») ; le diff est `git diff HEAD~1 HEAD` (**aplati, un seul diff**). Rapport de la P1 :
`/home/gcorbaz/devel/kesh-gate-logs/e506/review-p1.md` (lis-le **après** le diff). **Ce commit ne touche aucun code de production** : tests,
fiche, manuel `.tex`, prompt. Si le diff nomme un fichier de production, c'est un finding.

## La lentille unique : « chaque correctif tient-il, et n'a-t-il rien cassé ? »

Pour chacun des six findings de la P1, vérifie sur le diff **et** au code que la correction est réelle et complète :

- **R1-1** — `audit-log-page.test.ts` et `UnrejectTransactionAction.test.ts` épinglent `America/New_York` pour tout le fichier
  (`pinTimeZone` au niveau racine). Les hooks imbriqués des `describe` à `Europe/Zurich` (filets AC9) l'emportent-ils bien, et l'ordre
  `beforeEach`/`afterEach` rétablit-il le fuseau d'origine ? Un test non épinglé du fichier dépend-il du fuseau (une assertion de date) ?
  Le journal `/home/gcorbaz/devel/kesh-gate-logs/e506/mut-M1-bis.txt` montre-t-il **AC5 et AC6 rouges** (lis les lignes `FAIL`), et son
  `.diff.txt` la mutation exacte (retrait de `timeZone`) ? `reports.api.ts` est-il restauré (`git diff HEAD~1 HEAD --stat` ne le nomme pas) ?
  § *Mutations* et le Dev Agent Record disent-ils la même chose que le journal ?
- **R1-2** — AC11 n'est plus affirmé vert ; aucune autre ligne du Dev Agent Record n'affirme un gate non tombé (`testfast.log`,
  `e2e-complet.log` : lis-les, ils peuvent avoir changé).
- **R1-3** — plus aucune mention de `mut-garde.txt` (`grep -rnF "mut-garde" _bmad-output/implementation-artifacts/50-6-*`) ; M4 décrit comme variante en ligne.
- **R1-4** — l'angle mort `.getFullYear/.getMonth/.getDate/.getDay` est écrit dans l'en-tête de `local-date-guard.test.ts` **et** dans D3 de la
  fiche ; la liste est-elle exacte (les sites cités existent-ils : `premier-exercice.ts`, `fiscal-years.helpers.ts`, `mcp-time.ts`, `local-date.ts`) ?
- **R1-5** — les captures 09 (rejet de fin de soirée) et 10 (version décidée) existent dans `captures/{avant-ny,apres-ny,apres-zurich}/` :
  **lis les images** (`avant-ny/09` dit-elle « Rejetée le 24 nov. 2026 », `apres-ny/09` « 25 nov. 2026 », `apres-ny/10` une date dans la
  colonne *Décision* ?). La base est alignée entre AC13 et le Dev Agent Record (`kesh_e506b`).
- **R1-6** — la phrase de `docs/manual/fr/user-manual.tex` (vers `:790-799`) : phrases courtes, exacte **contre le code** (exception des dates
  saisies, limite des instants de l'écran *Serveur MCP* : `mcp-time.ts`, `McpKeysSection.svelte`, `McpSettingsSection.svelte`), et le contrôle
  du **texte aplati** tient toujours : `tr '\n' ' ' < docs/manual/fr/user-manual.tex | tr -s ' ' | grep -o "affiche l'heure de Zurich, quel que soit le fuseau de votre poste" | wc -l` → 1 ;
  l'ancienne phrase (« affiche l'heure de votre .textbf{poste} ») → 0. **Le manuel, PDF compris** : aplatis le PDF
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, ligature « ﬀ » : cherche `heure de votre poste`) ; il dit encore
  l'ancienne phrase jusqu'au `make fr` de la release (attendu, non commité) ; le `.tex` compile-t-il (accolades, `~;`, `\emph`) ?

**Propagation** : greper le symptôme de chaque correctif sur tout le dépôt (`heure du poste`, `mut-M1.txt`, `kesh_e2e_e506`, `huit captures`,
`8 captures`) et dire les sites oubliés.

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. Aucun `cargo`, `npm`,
`npx`, `node`, `make`, `docker`, `mariadb`, `sqlx`, `curl`, aucun `scripts/*` (en particulier ni `scripts/prepare-release.sh`, ni
`scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`), aucun lancement de Vitest ni de Playwright. Aucun `git commit`,
`git push`, `git checkout`, `git switch`, `git stash`, `git merge`, `git restore`, `git worktree`. Aucun `gh` qui écrit ; au plus trois
requêtes `gh` de lecture, espacées d'une minute. Rien sous `~/kesh/`. `pdftotext`, `python3 -I` en lecture admis.

## Rapport

Un seul fichier : `/home/gcorbaz/devel/kesh-gate-logs/e506/review-p2.md` : verdict chiffré CRITICAL / HIGH / MEDIUM / LOW ; chaque finding
(`R2-n`, sévérité, `chemin:ligne` et ligne copiée, correction) ; axes **exercés** et **non exercés** avec la raison (un « 0 finding » sans cette
liste ne compte pas) ; correctifs confirmés (une ligne chacun). Réponds ensuite par le verdict chiffré et le chemin du rapport.
