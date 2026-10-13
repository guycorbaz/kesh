# Prompt — validation P1 de la spec, Story 28-1 (les boutons d'action destructive se voient)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-28-1`, branche `story/28-1` (base `6f971d76`, = v0.13.2 + rétrospective de
l'Epic 27).
Fiche : `_bmad-output/implementation-artifacts/28-1-boutons-destructifs.md` (commit `bef56717`).
Epic : `/home/gcorbaz/devel/kesh-28-0/_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` (brouillon du
kickoff, § 28-1 et § *Recette visuelle (B2)* — lecture seule, autre worktree). Issue #640
(`gh api repos/guycorbaz/kesh/issues/640` et `…/640/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes-28-1.md` (C-28-1-1 à 16).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Issue Tracking Rule ; § Test Locally
First, E2E). Spec UX : `_bmad-output/planning-artifacts/ux-design-specification.md` l. 286, 446, 514, 639-662,
714-718.

⚠️ La fiche **corrige la prémisse de l'issue** (C-28-1-1) : selon elle, le jeton `destructive` n'est pas défini
dans `frontend/src/app.css` et Tailwind v4 n'engendre aucune règle pour `bg-destructive/10`, `text-destructive`…
Vérifie-le toi-même avant d'en tirer quoi que ce soit — `frontend/src/app.css`, et le build existant
`/home/gcorbaz/devel/kesh/frontend/build/_app/immutable/assets/*.css` (`grep -c destructive`, lecture seule ;
date du build à noter, il n'est pas celui du worktree). Si la prémisse est fausse, toute la fiche l'est : au
moins **CRITICAL**.

## Axes prioritaires (communs aux deux lentilles)

1. **Inventaire clos** (Dev Notes § Inventaire, classes A à E) : refais les commandes copiées dans la fiche et
   **élargis-les** — un détecteur part du symptôme, pas d'une liste. Le symptôme : *un contrôle qui déclenche une
   action destructive et ne se lit pas comme un bouton*, ou *une classe de couleur qui ne produit aucune règle*.
   Cherche : `variant="destructive"` sur une ligne à part (balise multi-lignes), `variant={…}` dynamique,
   `buttonVariants(` appelé ailleurs, les composants `ui/` (`dropdown-menu-item`, `select-item`, `input`,
   `select-trigger`), `<a>`, `role="button"`, `Dialog.Close`, `AlertDialog`, classes rouges passées par une
   variable de script ou par `cn(…)`, `style=`, `text-red-*`, `text-error`, `hover:text-destructive`,
   `class:text-destructive`. Un site destructif manquant ou mal classé est au moins **MEDIUM** ; une erreur dans
   la table fermée de l'AC6 (neuf entrées) l'est aussi.
2. **Mesures** : recalcule **chaque** ratio de contraste cité (fiche AC1, AC3, registre C-28-1-3 à 6 :
   6,47 / 8,31 / 5,45 / 4,59 / 2,53 / 3,68 / 4,13 / 4,83 / 5,58 / 3,76) à la main par la formule WCAG 2
   (luminance relative, `(L1 + 0,05) / (L2 + 0,05)`), sans interpréteur — montre le calcul d'au moins trois. Un
   ratio faux qui fait passer un seuil est **HIGH**.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` (l. 1725-1745, *Supprimer un import* ;
   l. 1107 ; l. 1245 et 2019, *Annuler le rapprochement*) et `docs/manual/fr/admin-manual.tex` l. 1985. Ce que le
   manuel dit **aujourd'hui** des boutons concernés est-il vrai au code ? Ce que l'AC10 prescrit sera-t-il vrai au
   code que la fiche prescrit (un bouton destructif désactivé est-il « désactivé » plutôt que « grisé » ; un autre
   passage du manuel décrit-il un bouton que l'AC5 transforme) ? Contrôle aussi les **PDF**, aplatis :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`). Une promesse
   que le code ne tiendrait pas est au moins MEDIUM.
4. **Recomptes** : 24 / 17 / 201 / 42 / 33 / 73 / 29 / 11 / dix specs axe / 25 emplacements de capture / 13 AC /
   T0-T10 / M1-M17 / C-28-1-1 à 16, et chaque numéro de ligne cité (`sed -n`, `grep -nF`).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #640 (« Attendu » : fond plein, texte contrasté, AA, survol / focus / désactivé, les 17
  sites, les dialogues avec *Annuler*, le mode sombre) et du § 28-1 de l'epic (« fond plein ou contour », « captures
  avant / après », « vérifier qu'aucun site n'employait la variante pour un texte d'erreur ») couverte par un AC
  testable ; chaque AC par une tâche, chaque AC par un test **qui rougirait** et chaque mutation M1-M17 par le test
  nommé (le test décrit la ferait-il vraiment rougir ?) ; chaque fait, chemin et numéro de ligne revérifié ; les
  textes prescrits (CHANGELOG AC12, manuels AC10, spec UX AC11) confrontés à ce que le code prescrit fera ; les
  gardes existantes qui changeront de sens ou rougiront — `e2e-selecteurs-traduits.test.ts`,
  `i18n-libelle-en-dur.test.ts`, `i18n-keys.test.ts`, `lint-i18n-ownership`, `crates/kesh-api/tests/textes_coherents.rs`,
  les dix specs axe, `fiscal-years-page.test.ts`, `reconciliation-cancel.spec.ts` — la fiche les nomme-t-elle ?
- **F — Faisabilité et pièges techniques (adversaire de la conception)** : Tailwind **4.2.2** (`frontend/node_modules/tailwindcss`,
  lecture seule) — un jeton `@theme` `--color-destructive-hover` produit-il bien `hover:bg-destructive-hover` ; le
  variant `dark` est-il `prefers-color-scheme` (C-28-1-7) ; la règle `:focus-visible` d'`app.css:62` l'emporte-t-elle
  vraiment sur `outline-none` (couches de cascade : où Tailwind v4 place-t-il `@import 'tailwindcss'` et le CSS
  écrit après) ? `tailwind-merge` 3 (`cn`) traite-t-il `text-destructive-foreground` comme une couleur et non une
  taille, sans écraser `text-sm` / `text-xs` de la base et des tailles ? `disabled:opacity-50` laisse-t-il le fond
  calculé à `rgb(185, 28, 28)` (AC3) ? La garde AC6 est-elle réalisable avec `svelte/compiler` 5 (`parse(…, { modern:
  true })` : noms des nœuds `RegularElement`, `Component`, `Attribute`, `ClassDirective`, positions `start`/`end`) et
  voit-elle `class={cond ? '…' : '…'}` ? Les scénarios E2E (a)-(d) sont-ils **déterministes** (focus clavier via
  `Shift+Tab`/`Tab`, `hover()`, `emulateMedia`, montage d'import partagé avec `bank-import-delete.spec.ts`, seed
  `with-company` et compte `account-row-<n>-archive-button` existant) et font-ils rougir M1, M2, M3, M6, M9, M10 ?
  Les captures prises **avant** les assertions survivent-elles à un échec (T0) ? Le changement de couleur des 201
  usages du jeton peut-il faire rougir une spec axe existante (un `text-destructive` sur un fond autre que blanc ou
  `/10`) ? Les cinq migrations de l'AC5 changent-elles un comportement (soumission de formulaire, `type`, focus,
  `disabled`) ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, `vite`, `vitest`, `playwright`, `docker`,
`curl` vers le backend, `bash` sur un script du dépôt, **tout interpréteur** (`python`, `python3`, `perl`, `ruby`,
`node -e`, `awk` à effet de bord), `gh` en écriture, toute requête SQL. Rien de `~/kesh-test/` (relevé réel de Guy)
ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `ls`, `wc`, `sort`, `uniq`, `git log`/`show`/`diff`/
`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
