# Story 36-15 — revue de code, passe 2 ciblée (une lentille, sur la remédiation de la P1)

*Prompt versionné le 2026-11-16 par l'agent de développement (Opus 5.5). Passe **ciblée** (CLAUDE.md, « La passe ciblée ») :
une seule lentille, contexte frais, sur le seul commit de code de la remédiation. Modèle : Sonnet, Opus ou Haiku (passe
ciblée, non première).*

## Cible

- Worktree `/home/gcorbaz/devel/kesh-36-15`, branche `story/36-15-compte-analytique`.
- **Commit de code de la remédiation : `62a0b8083`** (`git show 62a0b8083`). Le commit de fiche qui le suit ne porte que
  des comptes rendus (fiche, registre, `sprint-status.yaml`, ce prompt) : à lire pour vérifier qu'ils disent vrai, non à
  relire comme du code.
- Rapports de la P1 remédiés : `/home/gcorbaz/devel/kesh-gate-logs/e36-15/review-p1-{A,B,E}.md`. Change Log de la fiche,
  entrée « remédiation de la revue P1 » : chaque finding et ce qui en a été fait.
- Arbitrages de Guy du 2026-11-16 : B1-4 (de-CH « pro analytisches Konto », accusatif, libellés et noms de fichier),
  B1-5 (« Analytisches Standardkonto », « analytisches Hauptkonto » gardés, inscrits au glossaire avec leur sens).

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni `scripts/*.sh`
(`prepare-release.sh`, `test-fast.sh`, `gate-ci.sh` compris), ni `git commit|restore|checkout|stash`, ni `sed -i`, ni
redirection `>`, ni fichier temporaire. `grep` est une fonction vers `ugrep` : `/usr/bin/grep` ou `git grep`. Le seul
fichier écrit est le rapport, par l'outil Write : `/home/gcorbaz/devel/kesh-gate-logs/e36-15/review-p2-ciblee.md`.

## Ce que la lentille cherche — les défauts que la remédiation a pu introduire

1. **`litteraux_de_chaine` et `fin_du_bloc`** (`crates/kesh-api/tests/textes_coherents.rs`) : le saut `i += 3` d'un
   caractère échappé — `'\''`, `'\\'`, `'\n'`, `'\u{2013}'`, un `'\` en fin de source (indice au-delà de la longueur :
   panique possible sur une tranche ?) ; les sondes neuves (`['\'','"']` puis « projet », `'\\'`) et celle de
   `sans_modules_de_test_apparie_les_accolades` prouvent-elles le correctif (mutation `mut/rem-extracteur.log`) ?
2. **Exceptions bornées de G25** (`ExceptionG25`, `g25_non_excuses`) : le retrait de l'extrait (`replace` sur la ligne en
   minuscules) peut-il excuser à tort (extrait qui contient « projet » deux fois, extraits qui se recouvrent, ordre des
   retraits) ou rougir à tort ? Le compte `lignes` est-il juste pour chacun des 16 extraits (recompter :
   `/usr/bin/grep -ci '<extrait>' <fichier>` — attention, le test lit le `.tex` **sans ses commentaires**) ? La ligne
   entière n'est-elle réservée qu'à `README.md:218` ? Mutations `mut/rem-borne.log`, `mut/rem-extrait.log`.
3. **Motif anglais** `(?i)project(s|'s)?\b` : « Projected » épargné, composés vus ; un faux positif possible dans les
   valeurs en-CH d'aujourd'hui (`awk` sur les valeurs, sortie copiée) ?
4. **Garde Vitest** (`frontend/src/lib/shared/vocabulaire-analytique.test.ts`) : la liste fermée des dix clés hors relevé
   et `toHaveLength(39)` sont-elles justes et cohérentes avec `CLES` (ordre compris) ?
5. **`projects_e2e.rs`** : les quatre textes affirmés égalent-ils, à l'octet, ceux de `routes/projects.rs` ?
   `error.message` est-il bien le champ servi ?
6. **de-CH** : les six valeurs changées (`grep -n 'pro analytisches' crates/kesh-i18n/locales/de-CH/messages.ftl`), le
   test `slugify_max_keeps_the_analytical_report_types_whole`, `util.rs:46`, la fiche (D2) et le glossaire disent-ils la
   même chose ? Reste-t-il « analytischem » après *pro* quelque part (`git grep -n 'pro analytischem'`) ?
7. **Glossaire** : les deux lignes ajoutées en partie A et le compte « 84 » (commande écrite dans la note, à rejouer
   en lecture : `awk … | wc -l` est permis) ; le sens écrit (pas un compte du plan) est-il juste ?
8. **Commentaire `reports/+page.svelte:425-430`** et **note de version** de la fiche : disent-ils vrai sur le nom donné
   par le navigateur (`triggerDownload`, `reports.api.ts`) et sur ce qui change pour un client de l'API ?

Chaque finding : sévérité, fichier et ligne, sortie de commande copiée (`grep -nF`, `-F` obligatoire) pour toute
affirmation de présence ou d'absence, correctif proposé. Le rapport **déclare les axes exercés et non exercés** ; un
« 0 » sans cette liste ne compte pas.

## Gates déjà joués (ne pas relancer)

Au Dev Agent Record et au Change Log, avec leurs journaux : gate ciblé `cible-rem2.log`, `test-fast-ci-rem1.log`, Vitest
complet et `npm run check` de la remédiation, mutations `mut/rem-*.log`.
