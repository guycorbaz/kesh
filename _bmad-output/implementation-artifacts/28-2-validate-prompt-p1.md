# Prompt — validation P1 de la spec, Story 28-2 (une erreur d'API ne s'affiche plus « [object Object] »)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-28-2`, branche `story/28-2` (base `6f971d76`, = v0.13.2 + rétrospective de l'Epic 27).
Fiche : `_bmad-output/implementation-artifacts/28-2-erreurs-object-object.md`. Issue #520
(`gh api repos/guycorbaz/kesh/issues/520` et `…/520/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes-28-2.md` (C-28-2-1 à 14). Antécédent :
`15-5b-gardes-surfaces-neuves.md` (AC14, C-15-5b-1, et `:976-979`). Rétrospective :
`epic-27-retro-2026-10-13.md` § 7 (B2, recette visuelle) et § 9.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Test
Locally First, E2E ; § Issue Tracking Rule).

⚠️ La fiche **corrige l'issue** : `reports/+page.svelte:210` serait déjà gardé, et deux sites non cités par l'issue
rendraient encore « [object Object] ». Vérifie-le toi-même avant d'en tirer quoi que ce soit.
⚠️ La fiche **déroge** à la règle de découpage préventif (six modules, C-28-2-9) : juge la dérogation, ne la
présume ni juste ni fausse.

## Axes prioritaires (communs aux deux lentilles)

1. **Inventaire clos** (Dev Notes § *Inventaire des sites NON RÉSOLUS*) : refais les `grep` copiés et **élargis-les**
   (un détecteur part du symptôme, pas d'une liste) — autres noms de liaison (`caught`, `ex`, `reason`, `problem`,
   `_e`, `e2`), `catch` sans parenthèses de type (`catch (e: unknown)`), rappels `.catch(function (x)`, `.then(…, (x)
   => …)` (second argument de `then`), `Promise.allSettled` et `.reason` sous toutes graphies, `{:catch}` Svelte,
   `onerror`, `window.addEventListener('error'|'unhandledrejection')`, `+error.svelte` (`page.error`),
   `String(` dans le **balisage** Svelte, `toast.error(`/`notifyError(` recevant une valeur brute, concaténations
   `'…' + err`, `Object.prototype.toString`, `` `${err.message}` `` sur une valeur non gardée, `.message` lu après un
   cast `as Error` / `as ApiError` / `as any`. Un site affiché qui rend « [object Object] » et manque à la classe A
   est au moins **HIGH** ; un site de classe B manquant, au moins **MEDIUM**. Recompte la **classe D** (détecteur de
   la fiche) et ses deux faux positifs déclarés.
2. **Manuel contre code** (obligatoire) : la fiche affirme qu'aucun manuel ne décrit ces messages. Vérifie-le sur
   les `.tex` **et** sur les PDF aplatis (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`,
   idem `admin-manual.pdf`, et les manuels `de/en/it` s'ils existent) : « [object Object] », « Impossible de
   contacter », « Erreur serveur », « ne répond pas », « Erreur inattendue », le verrouillage des livres et les profils
   bancaires (ce que le manuel dit du message d'un refus). Une promesse que le code prescrit ne tiendrait pas est au
   moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — 317 fichiers, 257 `catch` (164 liés,
   93 non liés), 20 `.catch(` dont 5 liés, 26 lignes du premier `grep` et leur ventilation 3 + 3 + 20, 4 + 16 sites
   A/B, ≈ 23 sites D (25 − 2), 177 lectures de `.message`, 20 sites `error-unexpected`, 18 fichiers qui simulent le
   client, `sitesTotal` 2075 → 2079 et la ventilation +1 +3, les planchers de l'AC6 (300, 160 ; 169 mesurés),
   11 AC, T0-T10, M1-M20, C-28-2-1 à 14, et chaque numéro de ligne cité (`sed -n`, `grep -nF`).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #520 (constat **et** attendu : « le message du serveur est affiché ; un helper unique fait
  la conversion ») couverte par un AC testable, chaque AC par une tâche, chaque AC par un test **qui rougirait** ;
  chaque mutation M1-M20 confrontée au test qu'elle doit faire rougir (le test prescrit la verrait-il vraiment ?) ;
  chaque fait, chemin et numéro de ligne revérifié ; les textes prescrits (trois clés × 4 locales, CHANGELOG)
  confrontés au glossaire et aux traductions voisines (`auth-recovery-error-network`, `error-unexpected`) ; les
  gardes existantes qui changeront de sens — `i18n-keys.test.ts` (`sitesTotal`, `sitesNonResolus`, `relais`,
  `sitesGabarit`), `i18n-un-repli-par-cle.test.ts`, `i18n-entrees-a-variables.test.ts`,
  `i18n-libelle-en-dur.test.ts`, `e2e-selecteurs-traduits.test.ts`, `lint-i18n-ownership.js`,
  `crates/kesh-i18n` (`parity_between_locales`), `crates/kesh-api/tests/textes_coherents.rs` — la fiche les
  nomme-t-elle toutes, et ses prévisions sont-elles justes ? Les tests existants « pas « [object Object] » » du
  rapprochement restent-ils verts sans modification, une fois `errorMessageOf` déplacé et changé de comportement
  pour une `Error` (lis-les : un test qui attend le message d'une `Error` rougirait) ?
- **F — Faisabilité et conception (adversaire)** : le **cycle d'import** `api-client.ts` ↔ `i18n.svelte.ts`
  (C-28-2-4) est-il réellement bénin — sous Vite/SvelteKit (SSR compris : `+layout.ts`, `+page.ts` qui appellent le
  client côté serveur ?), sous Vitest, avec les 18 `vi.mock` du client et les éventuels `vi.mock` d'`i18n.svelte` /
  d'`onboarding.svelte` ? Un `ApiError` fabriqué **avant** le chargement du catalogue (au démarrage, sur
  `/api/v1/i18n/messages` lui-même) rend-il un texte sensé ? L'extraction d'`isApiError` en `api-error.ts` avec
  réexport change-t-elle quoi que ce soit pour un test qui simule le client **avec** un `isApiError` maison ? Le
  rendu générique d'une `Error` (C-28-2-3) masque-t-il un message **destiné à l'utilisateur** qu'un site affiche
  aujourd'hui (`throw new Error(…)` du code applicatif, `Error` levée par `download.ts`, `clipboard.ts`, un
  `res.json()` qui échoue, un `DOMException`) ? Le détecteur de l'AC6 est-il **décidable et sans faux négatif sur
  les vingt sites** (portée des liaisons par équilibrage des accolades dans un `.svelte` — accolades du balisage,
  gabarits imbriqués, chaînes contenant `{`/`}`, expressions régulières ; exemption `console` multi-ligne) ; ses
  planchers anti-muet détectent-ils un parcours vide **et** un recueil de liaisons cassé ? Les deux scénarios E2E
  sont-ils **déterministes** (interception avant navigation, rejeu GET et `__KESH_RETRY_DELAYS`, état dégradé
  `apiHealth` qui affiche peut-être une bannière à la place du message, lecture du catalogue servi) et font-ils
  rougir M9 et M11 ? La dérogation au découpage (C-28-2-9) tient-elle, et la classe D (C-28-2-10) est-elle bien un
  défaut **distinct** de #520 ou une part de l'« attendu » de l'issue ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-2-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-2-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/install-hooks.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/
`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, `deno`, `bun`,
`python`/`python3`, `perl`, `ruby`, `tsc`, `vitest`, `playwright`, `docker`, `bash`/`sh` sur un script du dépôt,
**tout interpréteur ou exécutable de script, quel qu'il soit**, `gh` en écriture (`pr`, `issue`, `comment`,
`api -X POST/PATCH/PUT/DELETE`), toute requête SQL, `sed -i`. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit
ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `find`, `wc`, `ls`, `git log`/`show`/`diff`/`blame`/`grep`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
