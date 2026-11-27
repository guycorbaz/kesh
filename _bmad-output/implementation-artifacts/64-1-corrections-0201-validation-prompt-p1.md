# Prompt — validation P1 de la fiche `64-1-corrections-0201` (#1178, #1758, #1712), passe complète

Tu valides, **en lecture seule**, la fiche de story `64-1-corrections-0201` de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes), seule story de l'Epic 64, visée **v0.20.1** (dimanche 2026-11-29 ou lundi 2026-11-30, **sans migration**).
Kesh tient les livres réels du Project Lead (Guy). Trois défauts : le formulaire des taux de TVA n'envoie rien (#1178, P2), la page
`/invoices/{id}/edit` s'ouvre sur une facture validée (#1758, P3), `PUT /api/v1/profile/mode` sans garde de rôle (#1712, P3).

- **Modèle : Opus. Contexte frais. Passe complète** (première passe : jamais ciblée, jamais Haiku). Langue du rapport : français.
- Worktree : `kesh-64-1` (à côté du dépôt principal), branche `story/64-1-corrections-0201`, partie de `epic/64-corrections-0201`
  (= `main` `0e99cd2db`). Fiche : `_bmad-output/implementation-artifacts/64-1-corrections-0201.md` ;
  epic : `_bmad-output/planning-artifacts/epic-64-corrections-0201.md`.
- Lis d'abord : le `CLAUDE.md` du worktree (Qualité du code, Recette visuelle, Test Locally First, Review Iteration Rule, Inventorier
  les sites NON RÉSOLUS, Propagation post-patch, Priorités des défauts, Issue Tracking Rule, Règle de splitting préventif, Versions
  de l'API, Migration breaking policy, Documentation à synchroniser) ; `docs/agents-orchestration.md` ; les trois issues
  (`gh issue view 1178|1758|1712 --json number,title,body,comments`, lecture seule).

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. En particulier, tu ne lances **ni** `cargo` (build, test, nextest, clippy, fmt),
**ni** `npm` / `npx` (check, test, build, lint), **ni** `make`, **ni** aucun `scripts/*` (dont `scripts/prepare-release.sh`,
`scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/recette-montee.sh`, `scripts/install-hooks.sh`) ni
`gate-distant.sh`, **ni** `docker` / `docker compose`, **ni** `mariadb` / `sqlx`, **ni** `curl` / `wget`, **ni** `git commit`,
`git checkout`, `git stash`, `git restore`, `git reset`, `git push`, `git worktree`, **ni** `gh` en écriture (`issue create|edit|comment|close`,
`pr …`), **ni** `sed -i`, `perl -pi`, `tee`, `>`, `>>` ou toute redirection vers un fichier (même « jetable »), **ni** `pkill`,
`killall` ni `kill` par motif, **ni** `python3` ou tout autre interpréteur (lire avec `grep`, `sed -n`, `cat`). Rien sous `~/kesh/`.
Permis : `git log`, `git show`, `git diff`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext … -` (vers la sortie standard,
**en tube**, jamais vers un fichier), `gh … view|list`.

Rapport : **`$GATE_LOGS/s64-1/validation-p1.md`** (le seul fichier que tu écris, par l'outil d'écriture).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Prémisses au code.** Chaque ligne citée par la fiche (§ *Verdict au code*) est relue à `0e99cd2db` : `vat-rates/+page.svelte`
   `:43`, `:124`, `:155`, `:287`, `:331`, `:147-166` ; `edit/+page.svelte` ; `invoices/[id]/+page.svelte:1068` ; `lib.rs:1374` ;
   `routes/profile.rs`. Les trois défauts sont-ils **réels** ? En particulier #1178 : le composant `Input`
   (`frontend/src/lib/components/ui/input/input.svelte`) et la source de **Svelte 5.55.1** installée
   (`frontend/node_modules/svelte/src/internal/client/dom/elements/bindings/input.js`, lue sans rien exécuter) rendent-ils bien un nombre
   (et `null` pour un champ vide) à `bind:value` sur un `type="number"` — y compris quand le `type` est une prop dynamique du composant ?
   La fiche affirme qu'un test d'essai l'a confirmé (non commité) : **le défaut peut-il être autre** (ex. `step`, `min`/`max`) ?
2. **#1178, le correctif (D1, D2, AC1-AC3).** `String(fRate).trim()` suffit-il : nombres à exposant (`1e-7`), `-0`, `NaN`, séparateur
   décimal selon la locale du navigateur (le `valueAsNumber` d'un `type="number"` en `fr-CH`), valeur `0` (falsy : `fRate == null`
   et non `!fRate`) ? L'ordre clôture-puis-création de `submitChange` : la validation **avant** `updateVatRate` couvre-t-elle tous les
   refus prévisibles (date de bascule antérieure à `validFrom` de l'ancien, taux hors 0-100) ou seulement le champ vide ? Un échec de
   `createVatRate` **après** la clôture reste possible (réseau, 409, chevauchement) : la fiche le dit-elle, `await load()` suffit-il ?
   Autres écrans du même défaut (`type="number"` + `bind:value` + `.trim()`) : la tâche 8 les **inventorie-t-elle comme ensemble clos**
   (grep de la forme *et* des variantes `Number(`, `parseFloat`, `+value`) ?
3. **#1758, le correctif (D3, D4, AC4, AC5).** Tous les statuts de `InvoiceStatus` sont-ils couverts ; un **avoir** passe-t-il par cette
   page (`credit-notes`) ? Le formulaire peut-il apparaître un instant avant la lecture du statut (`loading`, `invoice` initial `null`) ?
   Le même défaut existe-t-il pour `supplier-invoices/[id]/edit` (la fiche renvoie à une issue : est-ce juste, ou la story est-elle
   incomplète pour la même cause ?). Les clés i18n : conventions de nommage, propriété (`docs/i18n-key-ownership-pattern.md`,
   `frontend/scripts/lint-i18n-ownership.js`), quatre locales, `sitesTotal`.
4. **#1712, le correctif (D5, D6, AC6, AC7).** Déplacer la route dans `admin_routes` : **qui d'autre appelle `PUT /profile/mode`** (le
   frontend : `mode.svelte.ts`, `design-system/+page.svelte`, l'onboarding ; MCP ; scripts ; E2E ; seed) et que devient chacun ?
   `require_not_pat` : un appelant par clé d'API existe-t-il ? Le **test de garde** `admin_pat_denied_e2e.rs` (54 couples, marqueurs
   `KESH-ADMIN-ROUTES-BEGIN/END`, `nothing_is_added_after_the_first_route_layer`) : la fiche dit-elle **tout** ce qu'il faut changer
   (liste, décompte, phrase d'historique, autres assertions de source) ? `audit_route_registry.rs:784` code-t-il le groupe de droits ?
   La route reste-t-elle atteignable en `204` par l'Admin avec la même trace ? **Admin seul ou Admin + Comptable (Q1)** : donne ton
   avis motivé (effet réel du mode Expert, qui l'utilise, `docs/manual`). Le comportement du frontend pour un non-administrateur
   (`toggleMode` local + `PUT` silencieux + `syncModeFromServer`) : D6 est-il juste et suffisant ? Les `POST /onboarding/mode`
   (`routes/onboarding.rs`) : verrouillage par étape confirmé au code, ou un autre chemin d'écriture de `ui_mode` sans garde ?
5. **Versions de l'API (D7, Q2).** Applique § *Versions de l'API* du `CLAUDE.md` : une garde de rôle neuve sur une route existante
   non documentée est-elle une rupture, une correction « contraire au contrat documenté », ou un ajout neutre ? La route est-elle dans
   `docs/api-external.md`, dans la description OpenAPI (`grep -rn "profile/mode" docs crates frontend`) ? La phrase `:227` (« toute
   route `/api/v1/*` de l'UI est consommable ») change-t-elle la qualification ? Donne ton avis sur `c` (0.20.1) contre `b`.
6. **Critères et tests (AC1-AC9, tableau des tests).** Chaque AC est testable et rougit pour la bonne raison ; les **mutations** du
   tableau sont-elles chacune tuées par un test nommé ; sonde positive avant toute assertion d'absence (le formulaire absent, l'entrée
   de menu absente, l'écriture au journal absente) ; aucun test vrai par construction ; le test « base intacte » compare-t-il `version`
   avant/après ; E2E : la création d'un taux contre le vrai serveur laisse-t-elle un état qui casse `vat-rates.spec.ts` ou d'autres
   specs (taux actifs, `validTo` de l'ancien, ordre d'exécution) ? Une personne de rôle Comptable / Consultation : `createUser` existe-t-il
   dans `frontend/tests/e2e/helpers/api-fixtures.ts` et suffit-il ?
7. **Manuel, PDF compris.** Vérifie le manuel (`docs/manual/`) **contre le code**, **PDF compris**, aplati avant de greper :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `user-manual.pdf`) ; motifs : « Mode expert »,
   « Guidé », « taux de TVA », « Changer le taux », « Modifier » (facture), tableau des rôles. Le manuel dit-il aujourd'hui autre chose
   que le code sur les trois sujets ? DE/IT/EN existent-ils et la tâche 7 les nomme-t-elle tous ? `docs/api-external.md` : l'entrée
   prévue est-elle à la bonne place et cohérente avec `:227` et § 4 ? `README.md`, `website/` : mentionnent-ils le mode ou la page ?
8. **Règle de splitting et périmètre.** Recompte les modules au grain du domaine (crate / `features/<domaine>`) depuis la liste des
   fichiers touchés ; au-delà de cinq, finding. La fiche élargit-elle l'issue (D2, D6) au-delà de ce que décident les issues et le
   Project Lead ? Les trois correctifs sont-ils réellement indépendants ?
9. **Gates et processus.** Gates sur idefix (`$GATE_LOGS/serveur/gate-distant.sh <branche> tout` : branche
   poussée, usage lu dans le script) ; E2E complet au dernier commit de code ; recette visuelle de **chaque état touché** (TVA, page
   d'édition, menu utilisateur) ; PDF des manuels non commités ; aucune migration : rien n'oblige-t-il pourtant à toucher
   `post_restore.rs`, l'audit d'idempotence ou `kesh_version_min_required` ?
10. **Notes de version et documentation.** Puces par numéro de release, aucun epic nommé ; ⚠️ en tête pour la restriction ; aucun
    mot-clé de fermeture (`closes`, `fixes`, `resolves` et variantes) dans la fiche, l'epic ni les messages de commit
    (`git log origin/main..HEAD --format=%B`) ; `sprint-status.yaml` : clés `epic-64`, `64-1-corrections-0201`, statut, YAML valide
    (lis-le, ne l'exécute pas) ; l'epic est-il conforme au gabarit des epics voisins (`epic-54-totaux-tva.md`) ?
11. **Fichiers communs et coordination.** Conflits avec l'Epic 50 et les PR ouvertes (`gh pr list --state open`) sur
    `(app)/+layout.svelte`, `lib.rs`, `messages.ftl`, `i18n-keys.test.ts`, `sprint-status.yaml`.

## Rapport

Pour chaque finding : identifiant `V1-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée**
(`chemin:ligne` et l'extrait), conséquence, remède proposé. Pour tout CRITICAL ou HIGH qui affirme l'absence d'un code ou la
présence d'un anti-pattern, la commande `grep -nF` qui le prouve et sa sortie.
En fin de rapport : décompte **CRITICAL / HIGH / MEDIUM / LOW**, recompté depuis tes findings ; la liste des **axes exercés** et des
**axes non exercés** (avec la raison) ; les commandes lancées ; et ton avis sur **Q1 et Q2** de la fiche. Un « 0 finding » sans
cette liste ne compte pas comme passe.
