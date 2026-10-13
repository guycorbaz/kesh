# Prompt — revue de code P1, Story 28-1 (les boutons d'action destructive se voient)

*Versionné le 2026-10-13. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-28-1`, branche `story/28-1`. **Le diff à revoir** :
`git diff bbd0a9a1 3b0f7707 -- . ':(exclude)_bmad-output/implementation-artifacts'` (`bbd0a9a1` = validation
close de la fiche ; `37b74298` = dernier commit de code ; `3b0f7707` = PDF des manuels régénérés ; les commits
suivants ne portent que la fiche, le registre et ce prompt). ⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`, que le shell peut interpréter. La spec UX
(`_bmad-output/planning-artifacts/ux-design-specification.md`) est **dans** le diff. Fiche :
`_bmad-output/implementation-artifacts/28-1-boutons-destructifs.md` — AC1 à AC13, Tâches T0-T9, Dev Notes
(inventaire clos des classes A à E, angles morts 1 à 12) et le **Dev Agent Record** (phase rouge T0, choix
C-28-1-37 et suivants au registre `epic-28-choix-autonomes.md`, mutations M1-M22 sans M18, gates). Règles :
`CLAUDE.md` du dépôt — § « Recompter ses propres comptes rendus », § « Propagation post-patch », § « Le prompt d'une
passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses axes exercés ne compte pas », § « Inventorier les
sites NON RÉSOLUS ». Issues : #640 (la PR la fermera) ; #645, #646, #648 (référencées : angles morts assumés, non
corrigés ici).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt, sauf les captures nommées
ci-dessous ; la fixture E2E `frontend/tests/e2e/fixtures/csv_e2e_delete_import.csv` est synthétique.

**Contexte** : story **frontend seule**, trois modules de `frontend/src/` (AC5). `app.css` : trois jetons
`--color-destructive`, `-foreground`, `-hover` et leur commentaire de ratios (AC1). `button.svelte:14-20` : la
variante `destructive` à fond plein, sans `dark:`, bordure et anneau de focus à sa couleur, doc-comment (AC2).
`routes/design-system/+page.svelte` : `data-testid="ds-destructive"`, l'exemple désactivé, un carré de palette (AC9).
Gardes Vitest : `ui/button/button-destructive.test.ts` (AC7 (1)-(3)), `ui/jetons-non-definis.test.ts` (AC7 (4),
`@tailwindcss/node` déclaré, assertion de version, table fermée de treize noms), `ui/button/boutons-rouges-inventory.test.ts`
(AC6, `svelte/compiler`, quatorze entrées, (f)). E2E : `tests/e2e/destructive-buttons.spec.ts` (AC8),
`helpers/bank-import-fixtures.ts` (extraction), `bank-import-delete.spec.ts` (montage seul), `helpers/contrast.ts` et
`contrast.test.ts`. `frontend/package.json` et lockfile. Textes : manuels FR (`.tex` **et PDF**), spec UX,
`CHANGELOG.md` (`## [0.13.3] — Non publié`).

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :
1. **Le rendu réel.** Le jeton défini rend leur rouge à **tous** les usages du jeton (AC4) : bandeaux d'erreur
   (`border-destructive bg-destructive/10 text-destructive`), textes, champs `aria-invalid`, six `<button>` bruts.
   Un usage du jeton que le rouge neuf rend **faux** (un texte `text-destructive` sur un fond foncé, un `bg-destructive`
   sous un texte non blanc, un `bg-destructive/…` sans texte qui devient visible là où il ne devait pas) ?
   `grep -rnoE '(bg|text|border|ring)-destructive(/[0-9]+)?' frontend/src --include=*.svelte | grep -v '\.test\.'`.
2. **tailwind-merge et la base.** `cn(buttonVariants(...), className)` : la variante remplace-t-elle réellement
   `focus-visible:border-ring` et `focus-visible:ring-ring/50` ? Une classe de la base qui l'emporte encore (fond,
   texte, `aria-invalid`) ? Le rendu `<a href>` de la variante ?
3. **Les gardes prouvent-elles ce qu'elles nomment** (mémoire du projet : tests qui passent à vide, couverture
   supposée, montage qui ne construit pas l'état). AC7 (4) : la table a-t-elle été écrite **depuis la fiche** et non
   recopiée de la sortie (choix C-28-1-39, C-28-1-40) ? Un nom non défini qu'elle laisserait passer (`.js`, `<style>`,
   `@apply`, chaîne concaténée) ? AC6 : un `<button>` rouge qu'elle ne verrait pas (`{#snippet}`, `<svelte:element>`,
   attribut `class` dynamique, `Button.Root`) ?
4. **La recette E2E.** `softExpect` partout où une valeur est lue ; montage non `soft` ; captures dans le `finally` ;
   le pointeur éloigné avant le focus ; `:focus-visible` attendu ; dialogue refermé par *Annuler* ; deux tests par
   schéma, titres distincts ; base remise à zéro avant chaque test de (b). Le helper de capture qui avale une erreur
   (`testInfo.annotations`) : masque-t-il un défaut ?
5. ⛔ **LES MANUELS — `docs/manual/fr/user-manual.tex`, `admin-manual.tex` ET leurs PDF.** Les deux phrases
   « le bouton est désactivé » disent-elles **vrai** (`DeleteBankImportAction.svelte`) ? **Cherche une phrase des
   manuels, du README, du site (`website/`) ou de `docs/` que la story rend fausse** — une couleur, un « grisé », une
   description d'un bandeau d'erreur ou d'un bouton de suppression, *Annuler le rapprochement* « en lien ». Contrôle les
   **PDF aplatis** (`pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/`) — l'apostrophe y est typographique, écrire `.` à sa place dans un motif.
6. ⛔ **LA RECETTE VISUELLE — regarde les captures** avec l'outil de lecture d'images :
   `/home/gcorbaz/devel/kesh-gate-logs/28-1-captures/avant/*.png` et `…/apres/*.png` (douze chacune :
   `a-repos`, `a-survol`, `a-focus`, `a-desactive`, `b-bouton`, `b-dialogue` × `clair`/`sombre`). Le bouton « après »
   se reconnaît-il comme un bouton ? *Supprimer définitivement* se distingue-t-il d'*Annuler* ? L'anneau et le contour
   de focus figurent-ils sur `a-focus` (angle mort 10 : l'œil seul) ? Le désactivé se distingue-t-il du repos ? Le
   sombre est-il identique au clair ? Une capture tronquée ou vide ?
7. **CHANGELOG et spec UX** : une seule section `## [0.13.3] — Non publié` ; rien qui promette au-delà du code (les
   sites nommés deviennent-ils **vraiment** rouges ?) ; ratios cités = ceux d'`app.css` ; point décimal dans la spec UX.
8. **Décomptes** : 24 usages de la variante dans la fiche, **25** au `HEAD` (C-28-1-38) ; 14 entrées, 13 noms ;
   tests ajoutés ; Vitest / E2E / nextest déclarés au Dev Agent Record, chacun recompté depuis la source, avec son
   périmètre.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Le CSS engendré (cascade, `@layer`,
  `:focus-visible` hors couche contre `outline-none`), la variante en `<a href>`, en `disabled`, en `aria-expanded` ;
  les dialogues de la classe A (un fond de footer qui rendrait le bouton illisible) ; les bandeaux qui deviennent
  rouges dans un contexte non prévu (toast, tooltip `bg-foreground`, ligne de tableau) ; les deux helpers E2E et
  l'extraction (`FIXTURE_DIR` relatif à `helpers/`).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement les mutations
  M1 à M22 du Dev Agent Record et cherche une **mutation plausible qui resterait verte** (ex. : jeton défini hors de
  `@theme` ; `--color-destructive` en `oklch(…)` que la garde (2) ne lit pas ; un `!` important dans la variante ; un
  `dark:` réintroduit par `class` sur un site ; `outline-none` passé avant la règle globale ; une icône ✕ rendue rouge
  par un `style=`). Les auto-tests sur source synthétique couvrent-ils les branches du relevé ? L'E2E est-elle
  rejouable (deuxième passage sur la même base) et indépendante de l'heure ?
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (1 à 13) contre le code : satisfait,
  partiel, absent. AC5 (`git diff --stat bbd0a9a1 3b0f7707 -- frontend/src` : trois modules seulement). La
  phase rouge T0 est-elle documentée par une sortie et douze captures ? Chaque mutation exécutée, résultat rapporté,
  journal `kesh-gate-logs/28-1-mutations.log` ; les gates déclarés le sont-ils avec leurs chiffres et leurs échecs
  identifiés contre `docs/testing.md` § *Les échecs attendus* ? Les choix C-28-1-37 et suivants : justifiés, ou défaut ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-1-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu, ou capture regardée),
correction proposée. Pour tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) —
un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`,
`node`, `python3`, `python`, `deno`, `bun`, `tsx`, `ts-node`, `vitest`, `playwright`, `docker`, `gh` en écriture,
aucune requête SQL ni HTTP. Aucun fichier hors du dépôt, sauf les captures de `kesh-gate-logs/28-1-captures/`.
Autorisés : lecture (dont les images des captures), `grep`, `sed -n`, `wc`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
