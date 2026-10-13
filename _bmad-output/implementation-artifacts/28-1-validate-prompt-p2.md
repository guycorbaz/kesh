# Prompt — validation P2 de la spec, Story 28-1 (les boutons d'action destructive se voient)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet. **P2 : Opus**
pour les deux lentilles (R et F). Passe **complète**, non ciblée : la remédiation P1 a réduit le périmètre et
réécrit cinq AC.*

Dépôt `/home/gcorbaz/devel/kesh-28-1`, branche `story/28-1` (base `6f971d76`, = v0.13.2 + rétrospective de
l'Epic 27).
Fiche : `_bmad-output/implementation-artifacts/28-1-boutons-destructifs.md` (remédiation P1 au commit `d024f7ed` ;
diff de la remédiation : `git diff 12f64f95 d024f7ed`). Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes-28-1.md` (C-28-1-1 à 25 ; **C-28-1-17 à 25** sont
ceux de la remédiation, les entrées antérieures portent des rectifications ajoutées en italique). Rapports P1, à
lire pour savoir ce qui a été signalé : `/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p1-R.md` et `-F.md`.
Epic : `/home/gcorbaz/devel/kesh-28-0/_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` (§ 28-1 et
§ *Recette visuelle (B2)*, lecture seule, autre worktree). Issues #640, #645, #646 (`gh api
repos/guycorbaz/kesh/issues/<n>` et `…/<n>/comments`). Règles : `CLAUDE.md` (§ Review Iteration Rule et ses
sous-sections — le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le recompte
des comptes rendus ; § Règle de splitting préventif et sa dérogation ; § Issue Tracking Rule ; § Test Locally
First, E2E). Spec UX : `_bmad-output/planning-artifacts/ux-design-specification.md` l. 286, 446, 514, 639-662,
714-718.

## Le périmètre, tel que l'orchestrateur l'a arrêté

La 28-1 se limite à **#640** : jeton `destructive` (et `-foreground`, `-hover`), variante `destructive` du
`Button`, boutons rouges stylés à la main (inventaire, AC5), gardes de régression, recette visuelle. **Hors
périmètre, avec leur issue** : les autres jetons de couleur non définis → **#645** ; la suppression d'une personne
de contact sans confirmation → **#646**. Ne propose **pas** de les réintégrer : un finding qui revient à « il faut
définir `muted` » ou « il faut une confirmation » n'est pas recevable. Est recevable, en revanche, tout ce que la
fiche **affirme** sur ces sujets et qui serait faux, ou tout effet de la story sur eux qu'elle n'aurait pas vu.

## Axe premier — les régressions de la remédiation P1

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* (CLAUDE.md, § La passe ciblée). Lis
d'abord `git diff 12f64f95 d024f7ed`, puis, pour chaque choix C-28-1-17 à 25 :

1. **AC2 / C-28-1-20** : la variante gagne `focus-visible:border-destructive`. tailwind-merge 3.5
   (`frontend/node_modules/tailwind-merge`) remplace-t-il vraiment `focus-visible:border-ring` de la base (même
   groupe de conflit, même modificateur) ? La fusion de tailwind-variants passe-t-elle par tailwind-merge pour
   cette paire ? Le style calculé prescrit au focus (`border-top-color: rgb(185, 28, 28)`) est-il celui que
   Chromium rendra — la base porte `border border-transparent` : la largeur est-elle bien 1 px ?
2. **AC7 / C-28-1-18** : `buttonVariants.variants.variant.destructive` existe-t-il, avec ce chemin exact, dans la
   version installée de tailwind-variants (`frontend/node_modules/tailwind-variants`, `dist/`, et sa déclaration
   de type) ? Les assertions (1), (1-bis), (2) rougiraient-elles **au premier lancement** sur le code prescrit
   (faux rouge), ou passeraient-elles à vide ? M1, M3, M6, M8, M19 sont-elles tuées ?
3. **AC7 (4) / C-28-1-23** (facultatif) : `compile`, `build`, `__unstable__loadDesignSystem`, `candidatesToCss`
   existent-ils dans `@tailwindcss/node` / `tailwindcss` installés, avec ces signatures ? `@import 'tailwindcss'`
   d'`app.css` se résout-il sans le plugin Vite (option `base`, `loadStylesheet`) ? Un candidat comme
   `text-text-muted`, `border-b`, `ring-3`, `outline-none`, `bg-clip-padding`, `accent-…` est-il résolu (donc hors
   table) ? La liste attendue de douze noms est-elle **juste** — refais le relevé indépendant par `grep` (au moins
   un nom de plus ou de moins = MEDIUM) ; la règle « la table ne se remplit pas depuis la sortie de la garde » est-
   elle praticable ? M20 est-elle tuée dans ses deux sens ?
4. **AC6 / C-28-1-22** : la clé `chemin#data-testid` / `chemin@élément:rang` est-elle unique pour les neuf
   entrées **et** décidable (que vaut le « texte source » d'un `data-testid="api-keys-revoke-{k.id}"` dans l'AST
   de `svelte/compiler` 5 : un `Attribute` dont la valeur est une séquence `Text` + `ExpressionTag`) ? La seconde
   table (sites migrés) est-elle réalisable — retrouver un `Component` `Button` par un `data-testid` dynamique
   (`contact-person-delete-{p.id}`) ? M18 (deux essais) est-elle tuée ?
5. **AC5 / C-28-1-21** : *Annuler le rapprochement* du détail d'import passe à `outline size="sm"`. Est-ce
   vraiment la forme de `InvoiceSettlements.svelte:106-111` ? Le manuel (`user-manual.tex:1245`, `:2019-2020`)
   reste-t-il vrai ? Le changement est-il cohérent avec ce que la fiche dit ailleurs (AC4, AC12, classe D,
   « déclencheurs neutres », CHANGELOG) ? La décision affaiblit-elle la correction de #640 sur l'écran même de la
   recette (bordure à 1,23:1, angle mort 9) au point de ne plus corriger l'issue ? Si oui, au moins MEDIUM,
   argumenté.
6. **AC8 / C-28-1-19** : `toHaveCSS` et `expect.poll` sont-ils prescrits partout où une transition est en cours ?
   L'assertion (e) de `reconciliation-cancel.spec.ts` est-elle juste — `border-top-width` vaut-il `0px`
   aujourd'hui (préflight `tailwindcss/preflight.css:15`) et `1px` après ; `rgb(226, 232, 240)` est-il la valeur
   de `--color-border` ; l'insertion avant `:139` casse-t-elle le scénario existant ? `parseRgb` strict fait-il
   rougir M2 proprement (fond `rgba(0, 0, 0, 0)`) ?
7. **C-28-1-24, dérogation de découpage** : le recompte des modules est-il juste (sept) ? La dérogation
   satisfait-elle la règle telle qu'écrite au CLAUDE.md (section `Dérogation règle de splitting`, justification,
   risque accepté) ? Son risque accepté est-il vrai (quels tests existent réellement pour les cinq sites) ?
8. **C-28-1-25, décomptes** : refais chaque nombre rectifié — 32 bandeaux, 4,57, 5,45 / 5,46, 19 / 16 / 3 specs
   axe (nomme-les), 11 `aria-invalid` hors `ui/`, 8 `warning-soft`, 3 `error-soft`, 2 `danger`, 1,23:1 — et le
   bilan « 2 / 9 / 5 » déclaré pour la lentille R de P1.

## Axes communs (repris de P1, sur le périmètre réduit)

1. **Inventaire clos** (Dev Notes, classes A à E, « déclencheurs neutres », « confirmation non destructive ») :
   refais les commandes de la fiche et élargis-les — *un détecteur part du symptôme, pas d'une liste*. Symptôme :
   *un contrôle qui déclenche une action destructive et ne se lit pas comme un bouton*. Un site manquant ou mal
   classé est au moins MEDIUM ; une erreur dans la table fermée de l'AC6 (neuf entrées) ou dans la table des sites
   migrés (cinq) l'est aussi.
2. **Mesures** : recalcule à la main (formule WCAG 2, sans interpréteur) chaque ratio cité — 6,47 / 8,31 / 5,45 /
   2,53 / 3,68 / 4,13 / 4,83 / 5,58 / 3,76 / 1,23 / 14,6 —, en montrant au moins trois calculs. Un ratio faux qui
   fait passer un seuil est HIGH.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` (l. 1725-1745, 1107, 1245, 2010, 2019)
   et `admin-manual.tex` l. 1985, et les **PDF** aplatis (`pdftotext <pdf> - | tr '\n' ' ' | tr -s ' '`). Ce que
   l'AC10 prescrit sera-t-il vrai au code prescrit ? Un passage du manuel décrit-il un bouton que l'AC5 transforme
   (en particulier *Annuler le rapprochement*) ?
4. **Recomptes** : 24 / 17 / 201 / 42 / 32 / 73 / 29 / 11 / 19-16-3 / 25 emplacements de capture / 13 AC / T0-T10
   / M1-M20 / C-28-1-1 à 25 / sept modules, et chaque numéro de ligne cité (`sed -n`, `grep -nF`).
5. **Cohérence interne** après remédiation : greppe chaque valeur retirée ou changée (`4,59`, `33`, `dix specs`,
   `Issue à ouvrir`, `M1-M17`, `C-28-1-16` comme borne, `destructive" size="xs" class="ml-2"` pour le site
   `bank-import/[id]`, `cinq modules`) — elle ne doit survivre que dans le Change Log de P1 et les rectifications
   du registre. Une contradiction entre deux AC, entre un AC et le registre, ou entre la fiche et #645/#646 est au
   moins MEDIUM.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase « Attendu » de #640 et du § 28-1 de l'epic couverte par un AC testable **dans le périmètre
  réduit** ; chaque AC par une tâche et par un test qui rougirait ; chaque mutation M1-M20 par le test nommé ;
  chaque fait, chemin et numéro de ligne revérifié ; les textes prescrits (CHANGELOG AC12, manuels AC10, spec UX
  AC11) confrontés au code prescrit ; les gardes existantes nommées (`e2e-selecteurs-traduits.test.ts`,
  `i18n-libelle-en-dur.test.ts`, `contacts-page.test.ts:729`, les specs axe, `fiscal-years-page.test.ts`,
  `reconciliation-cancel.spec.ts`, `bank-import-delete.spec.ts`) ; la cohérence des renvois à #645/#646 avec le
  texte de ces issues et avec le commentaire posté sur #640.
- **F — Faisabilité et pièges techniques (adversaire de la conception)** : Tailwind **4.2.2**, tailwind-merge
  **3.5**, tailwind-variants, `svelte/compiler` **5**, `@tailwindcss/node`, Playwright (`toHaveCSS`,
  `expect.poll`, `emulateMedia`, focus clavier par `Shift+Tab`/`Tab`), tous en lecture seule sous
  `frontend/node_modules/`. Axes 1 à 4 et 6 de l'axe premier en priorité ; puis le déterminisme des scénarios
  (a)-(e), la survie des captures à un échec (T0), et tout effet des cinq migrations sur un comportement
  (soumission de formulaire — `ContactPersonsManager` est dans le `<form>` de `contacts/+page.svelte:824-1116` —,
  `type`, focus, `disabled`).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; indique pour chacun s'il **vient de la
remédiation P1** (`git diff 12f64f95 d024f7ed`) ou de la conception d'origine. Toute affirmation de présence ou
d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans
elle ne compte pas comme passe. Bilan par sévérité **recompté sur tes propres titres** avant d'être écrit (en P1,
le bilan déclaré d'une lentille ne correspondait pas à ses titres). Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, `vite`, `vitest`, `playwright`, `docker`,
`curl` vers le backend, `bash` sur un script du dépôt, **tout interpréteur** (`python`, `python3`, `perl`, `ruby`,
`node -e`, `awk` à effet de bord), `gh` en écriture, toute
requête SQL. Les questions de faisabilité (axe premier 2, 3, 4) se tranchent **par lecture** du code des paquets
installés, non par exécution : écris « non exécuté, lu à `fichier:ligne` ». Rien de `~/kesh-test/` (relevé réel de
Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `ls`, `wc`, `sort`, `uniq`, `git log`/`show`/
`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
