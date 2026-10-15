# Prompt — validation P1 de la spec, Story 29-9 (trois petits correctifs : #649, #655, #646)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-29-9`, branche `story/29-9` (base `fcd2ae75`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-9-trois-petits-correctifs.md` (commit `3176e1be`).
Registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-9.md` (C-29-9-1 à 12).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-9*, § *Ordre*, § *Releases*.
Issues (l'objet) : `gh api repos/guycorbaz/kesh/issues/649`, `…/655`, `…/646`.
Fiches **non mergées** qui partagent `ReconciliationProposals.svelte` et le manuel, en lecture seule :
`/home/gcorbaz/devel/kesh-29-3/_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`,
`/home/gcorbaz/devel/kesh-29-6/_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md`,
`/home/gcorbaz/devel/kesh-29-8/_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md`.
Précédent de forme : `_bmad-output/implementation-artifacts/28-8-petits-correctifs.md`.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Issue
Tracking Rule) ; `docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre
minimal ; aucune dette assumée sans issue ; aucun mot-clé de fermeture dans un commit).

## Axes prioritaires (communs aux deux lentilles)

1. **Prémisses** (§ *Le constat, au sol* et § *Les prémisses des issues*) : revérifie chacune au code de `fcd2ae75`.
   En particulier la rectification de #655 cas 2 (C-29-9-8) : est-il **vrai** qu'aucun chemin de l'application ne
   rejoue l'`$effect` de `bank-import/profiles/[id]/+page.svelte` (liens, `goto`, historique du navigateur, formulaire
   qui navigue) ? Et que la suppression d'une personne est un archivage sans retour à l'écran ? Une prémisse mal
   établie est au moins **MEDIUM**.
2. **#649, la conception** (AC1-AC4, C-29-9-2 à 6) : le palier unique « jusqu'à 500 » tient-il ? L'argument du coût
   (« proportionnel aux transactions rendues, pas au `limit` ») est-il vrai au code de `get_proposals` ? L'analyse de
   l'`$effect` qui suit `limit` est-elle juste en Svelte 5 (lecture synchrone avant le premier `await` dans `load()`
   appelée depuis l'effet) — et le remède (« le bouton ne fait que poser `limit` ») laisse-t-il une double requête
   ailleurs (`onAccept`, `onReject`, `onManualSuccess`, `onSplitSuccess` appellent `load()` sans changer `limit`) ?
   L'avis au plafond dit-il vrai (tri, filtre `pending` non rejetées) et a-t-il raison de ne pas nommer le rejet ?
   La sélection vidée (C-29-9-5) contredit-elle une phrase du manuel ou d'une fiche voisine ? Le reflet des constantes
   (C-29-9-4) : les deux tests Rust prescrits figent-ils vraiment ce que le frontend suppose ? Un défaut de conception
   qui laisse l'écran dire faux est au moins **MEDIUM**.
3. **#655, la conception** (AC5, AC6) : la garde de séquence couvre-t-elle **toutes** les écritures d'état après
   l'`await` (`detail`, `errorMessage`, `loading` ; `profile`, `errorMessage`, `isLoading`) ? Les écritures en tête de
   l'`$effect` du profil risquent-elles une boucle d'effet (`effect_update_depth_exceeded`) ? `isLoading = true`
   suffit-il à remonter `BankProfileForm` ? La fixture `$state` en `.svelte.ts` servie par `vi.mock('$app/state')`
   est-elle faisable avec la configuration Vitest du dépôt (`frontend/vite.config.ts`) ?
4. **#646, la conception** (AC7, C-29-9-10) : `window.confirm` dans un dialogue bits-ui ouvert — un piège de focus,
   d'`onclick` propagé, ou de fermeture du dialogue au retour du `confirm` ? Le texte de confirmation dit-il vrai
   (« aucun écran ne permet de la rétablir ») — cherche une route ou un écran qui modifierait `active` d'une personne
   (`PUT /api/v1/contact-persons/{id}`).
5. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` l. 869-874 et 2043-2060 ; ce que le
   manuel dit **aujourd'hui** est-il vrai au code de `fcd2ae75`, et ce que l'AC8 prescrit le sera-t-il au code
   prescrit ? Contrôle aussi le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s
   ' '`. Même exigence pour `docs/api-external.md:594`, les clés i18n prescrites (quatre locales) et
   `reconciliation-unreject-success`. Un texte qui promettrait ce que le code ne tiendrait pas est au moins **MEDIUM**.
6. **Sites non résolus et propagation** : refais les `grep` du § *Propagation* et **élargis-les** — tout texte (code,
   doc-comment, manuel, `api-external.md`, i18n ×4, fallbacks Svelte, tests, `README.md`, `website/`) qui affirme la
   limite de 100 sans suite, l'absence d'interface pour `hasMore`, ou la suppression sans confirmation. Un site
   oublié est au moins **MEDIUM**.
7. **Zones partagées** : confronte le § *Zones partagées* aux trois fiches voisines. La 29-9 casse-t-elle un test, un
   invariant ou un texte prescrit par elles (état (a) de la 29-6 ; paragraphe de priorité de la 29-3 ; bouton de ligne
   et modale de la 29-8 ; phrases du manuel `:2043-2060` qu'elles réécrivent) ? L'emplacement « après `</table>` »
   est-il libre dans les trois ?
8. **Découpage et périmètre** : la dérogation (C-29-9-1) est-elle juste au regard du § *Règle de splitting
   préventif* ? Chaque issue est-elle **entièrement** réglée, ou ce qui en reste est-il écrit comme exclusion avec une
   issue demandée ? La limite au-delà de 500 (C-29-9-3) est-elle une dette **cachée** (rétro 28 § 4.3) ?
9. **Recomptes** : chaque nombre et numéro de ligne de la fiche se recompte depuis la source — AC (10), tâches
   (T0-T9), tests Vitest (V1-V11), mutations (M1-M12), choix (12), clés i18n neuves (4) et réécrite (1), modules (4),
   `sitesTotal` au sol (2107).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase « Attendu » des trois issues couverte par un AC testable, chaque AC par une tâche, chaque AC par un
  test **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`, `grep -nF`) ; les
  registres et gardes que les changements doivent rejoindre (`i18n-keys.test.ts`, `lint-i18n-ownership`,
  `i18n-harvest.test.ts`, `boutons-rouges-inventory.test.ts`, `crates/kesh-api/tests/textes_coherents.rs`) — la fiche
  les nomme-t-elle tous ? Les patrons cités existent-ils tels quels (`loadGen`, `capturePath`, `vi.stubGlobal`,
  `seed_bank_transactions`, `make_new_tx`) ?
- **F — Faisabilité et exactitude de la conception (adversaire)** : axes 2, 3 et 4 en profondeur ; faisabilité des
  E2E (import CAMT de 501 transactions par l'API : taille, contrôle de doublons, dates antérieures aux autres fixtures,
  durée du calcul de 500 propositions ; dialogue natif dans Playwright) ; les mutations M1-M12 (chacune **tuable** par
  le test nommé ? M3 l'est-elle vraiment par un compte d'appels, alors que `loadGen` jette la réponse périmée ? en
  manque-t-il une — `hasMore` non remis à `false` quand une relecture n'en a plus, garde de séquence sur
  `loading`/`isLoading`) ; le test Rust de 501 transactions tient-il dans le temps du gate.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`),
`make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach` compris)/`switch`/
`restore`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, `vitest`,
`playwright`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui
écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Aucune donnée réelle de Guy ne se lit
ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture,
`pdftotext` vers la sortie standard.
