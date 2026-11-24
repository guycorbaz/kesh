# Prompt — validation P1 de la fiche 50-6 « Dates et heures de Zurich » (#1511), passe complète

Tu valides la **fiche** de la story **50-6** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), Epic 50 « Corrections »,
release **0.20.0** (coupure le vendredi 2026-11-27 au matin). Kesh tient les livres réels du Project Lead. **Aucune migration, aucun code
serveur** dans la story. Première passe : **complète**, contexte frais, modèle **Sonnet**. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-50-6`, branche `story/50-6-dates-heure-de-zurich`, partie de `origin/epic/50-corrections` à
`e5654fc00`. Fiche : `_bmad-output/implementation-artifacts/50-6-dates-heure-de-zurich.md` (statut `drafted`). Journal d'inventaire
(lecture seule) : `/home/gcorbaz/devel/kesh-gate-logs/e506/inv-symptome.txt`.

**Le point le plus important** : la fiche affirme, à son « Étape 0 », que la prémisse de l'issue est **fausse** (les quatre sites lisent
des horodatages qui portent `Z`, ou à qui le code ajoute `Z`) et que le seul défaut restant est l'affichage au **fuseau du poste** au lieu
de Zurich. Vérifie-le toi-même, au code, avant tout le reste ; ne le prends pas pour acquis.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Qualité du code*, *Recette visuelle*, *E2E (Playwright)*, *Règle de splitting préventif*, *Review
   Iteration Rule* (dont *Inventorier les sites NON RÉSOLUS* et *Le prompt d'une passe doit NOMMER le manuel*), *Priorités des défauts*.
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-fiche.md` (forme attendue d'une fiche de la 0.20.0).
2. Le fichier d'épique `_bmad-output/planning-artifacts/epic-50-corrections.md` (la 50-6 n'y figure pas encore : l'orchestrateur l'y
   ajoutera ; ne le compte pas comme un défaut de la fiche, mais dis si la fiche contredit l'epic, par exemple sur la borne de six stories).
3. Les issues, **en lecture seule**, avec **au moins une minute entre deux requêtes `gh`** (règle « Ménager les comptes GitHub ») :
   `gh issue view 1511 --json number,title,labels,milestone,body`, puis, une minute après au moins, `gh issue view 880 …`, puis `889`.
   Pas plus de trois requêtes `gh` dans toute la passe.
4. `/home/gcorbaz/devel/kesh-gate-logs/dry-audit/rapport.md` § D1 (l'origine de l'issue).
5. La fiche en entier.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — l'étape 0, relue au code serveur** : pour chaque champ (`createdAt` de l'audit, `autoMatchRejectedAt`, `lastAttemptAt`,
  `lastSuccessAt`, `nextDueAt`, `modifiedAt`, champs des envois et des prompts MCP), le type et la sérialisation sont-ils ceux que la
  fiche cite (`crates/kesh-api/src/routes/audit_log.rs`, `bank_imports.rs`, `backup_schedule.rs`, `mcp_envois.rs`, `mcp_prompts.rs`) ?
  Le DTO cité est-il bien celui que l'écran reçoit (suis la route jusqu'au type TypeScript) ? La sérialisation `serde` de
  `DateTime<Utc>` par chrono 0.4.45 finit-elle par `Z` (`~/.cargo/registry/src/*/chrono-0.4.45/src/datetime/serde.rs`) ? Un champ
  aurait-il un `#[serde(with = …)]` ou un `serialize_with` qui change la forme ? Si l'un des champs est naïf **et** lu sans `Z`, la
  fiche se trompe : c'est un HIGH.
- **A2 — prémisses relues au code frontend** : chaque `chemin:ligne` cité est-il juste à `e5654fc00` (ligne copiée) ? `zurichParts`,
  `formatSwissDate`, `formatSwissDateTime`, `mcp-time.ts`, `local-date-guard.test.ts`, `pin-time-zone.test-helper.ts`.
- **A3 — inventaire par le symptôme** : rejoue les commandes de `inv-symptome.txt` et cherche ce qu'elles **omettent** : un instant
  affiché par une autre forme (`toISOString()` montré brut, `.slice(11, 16)` pour l'heure, `Intl.RelativeTimeFormat`, un gabarit Svelte
  qui affiche une chaîne `…At` brute, une fonction d'un autre module qui formate un instant, un composant partagé de `lib/components/`
  ou `lib/shared/` qui formate une date). L'ensemble est-il **clos** : chaque site « en défaut », « juste » (raison vérifiable) ou « hors
  sujet » ? Le classement « juste » de chaque ligne tient-il (en particulier `fiscal-years.helpers.ts:31-32`, `api-keys/+page.svelte:72`,
  `users/+page.svelte:276`) ?
- **A4 — décisions** : chaque D a-t-elle son option écartée, motivée ? D0 est-elle correctement soumise au Project Lead (Q1), au vu du
  manuel qui documente l'heure du poste ? D1 (formateur dans `reports.api.ts` plutôt qu'un `lib/shared/utils/dates.ts`) est-elle juste au
  regard de la règle DRY **et** de la règle de splitting ? Une troisième copie de la lecture « naïf = UTC » subsiste-t-elle quelque part ?
- **A5 — la garde (D3)** : n'est-elle pas vraie par construction ? Les deux regex attrapent-elles toutes les formes (appel sur plusieurs
  lignes, `new Intl.DateTimeFormat` aliasé, `Reflect.construct`, `new globalThis.Date(x)`) ? Les angles morts sont-ils écrits ? La sonde
  positive et la sonde réelle (M4) prouvent-elles quelque chose ? L'allowlist annoncée close l'est-elle (compte-la contre l'inventaire) ?
- **A6 — AC constatables** : chaque AC dit-il un test, une valeur attendue, un fichier ? Les chaînes attendues d'AC2/AC3
  (`'25 nov. 2026, 00:30'`, `'15 juil. 2026, 00:30'`, bascules de mars et d'octobre 2026) sont-elles justes (dates des changements d'heure
  en Europe en 2026 : dernier dimanche de mars et d'octobre) ? Calcule-les par raisonnement, sans lancer Node.
- **A7 — preuve du rouge** : chaque test neuf rougit-il sans le correctif (ou par une mutation nommée, un journal chacune) ? AC9 est-il
  bien déclaré comme filet, non comme preuve ? Le piège du `$derived` créé avant l'épinglage du fuseau (P3) est-il traité ?
- **A8 — manuel, PDF compris** : `docs/manual/fr/user-manual.tex:4686-4691` et `:790-791`, `admin-manual.tex:2507-2509`, § `sec:mcp-envois` ;
  cherche dans les `.tex` **et** les PDF (`pdftotext docs/manual/fr/<f>.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o -i '.\{120\}heure.\{120\}'`,
  et `poste|fuseau|navigateur|UTC|Zurich`) toute phrase que la story rend fausse ; les langues `de`, `en`, `it` existent-elles vraiment
  (la fiche dit que non) ? La phrase du tableau de l'audit est-elle dans le PDF (la fiche dit ne pas l'avoir trouvée) ?
- **A9 — E2E et recette visuelle** : l'E2E d'AC11 est-il tenable (une action qui écrit une entrée d'audit, `timezoneId` par test,
  garde des specs de `local-date-guard.test.ts` qui interdit certaines constructions de dates) ? La recette d'AC13 couvre-t-elle chaque
  état visible touché (bandeau des sauvegardes, détail d'un envoi, prompts MCP) ?
- **A10 — splitting et modules** : le décompte de cinq modules est-il juste (un module = un crate ou un domaine du frontend dont le code
  de production change) ? `mcp-prompts` change-t-il de code ?
- **A11 — priorité, note de version, Dev Notes** : la requalification P2 → P3 proposée est-elle fondée au tableau des priorités ? La
  note de version nomme-t-elle une release, jamais un epic, et `refs #1511` sans mot-clé de fermeture ? Les issues I1, I2 sont-elles fondées
  au code ? La charge estimée est-elle crédible pour une clôture avant vendredi matin ?

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. Aucun `cargo`,
`npm`, `npx`, `node`, `make`, `docker`, `mariadb`, `sqlx`, `curl` vers un serveur, aucun `scripts/*` (en particulier ni
`scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni `scripts/recette-montee.sh`).
Aucun `git commit`, `git push`, `git checkout`, `git switch`, `git stash`, `git merge`, `git worktree`. Aucun `gh` qui écrit (`issue
create/edit/comment/close`, `pr …`, `api` avec `-X`/`-f`), et **au moins une minute entre deux requêtes `gh`**. Rien sous `~/kesh/`.
`python3 -I` en lecture pour recompter est admis.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e506/validation-p1.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant (`V1-n`), sévérité, axe, `chemin:ligne` et la ligne copiée qui le fonde, ce qui est faux, la correction
  proposée ;
- la liste des axes **exercés** (ce que tu as relu, commande lancée) et **non exercés** (avec la raison) — un « 0 finding » sans cette
  liste ne compte pas comme passe ;
- les prémisses de la fiche que tu as **confirmées** au code (une ligne chacune, ligne citée).

Rends ensuite, en réponse, le verdict chiffré et le chemin du rapport.
