# Prompt — validation P3 de la spec, Story 28-1 (les boutons d'action destructive se voient)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; P2 : Opus.
**P3 : Sonnet** pour les deux lentilles (R et F). Passe **complète**, non ciblée : la remédiation P2 a réduit le
périmètre, réécrit cinq AC (AC5 à AC9) et retouché six autres (AC1, AC3, AC4, AC10 à AC12), les tâches,
les mutations et les Dev Notes.*

Dépôt `/home/gcorbaz/devel/kesh-28-1`, branche `story/28-1`, **rebasée** sur `5aad8b35` (kickoff de l'Epic 28,
#643 ; = v0.13.2 + rétrospectives des Epics 27 et 28-0).
Fiche : `_bmad-output/implementation-artifacts/28-1-boutons-destructifs.md` (remédiation P2 au commit `f21d2db8` ;
diff de la remédiation : `git diff 3db1f345 f21d2db8 -- _bmad-output/implementation-artifacts/28-1-boutons-destructifs.md`).
Registre : `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md`, section *Story 28-1* (C-28-1-1 à
31 ; le registre séparé de la story y a été versé par union en P2 ; **C-28-1-26 à 31** sont ceux de la remédiation
P2 ; les entrées antérieures portent des rectifications en italique, « Rectifié / Remplacé / Annulé en validation
P2 »). Rapports P1 et P2, à lire pour savoir ce qui a été signalé :
`/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p1-{R,F}.md` et `28-1-validate-p2-{R,F}.md`.
Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` (§ 28-1 et § *Recette visuelle (B2)*). Issues
#640, #645, #646 (`gh api repos/guycorbaz/kesh/issues/<n>` et `…/<n>/comments`). Règles : `CLAUDE.md` (§ Review
Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation
post-patch, le recompte des comptes rendus ; § Règle de splitting préventif et son amendement D5 ; § Issue
Tracking Rule ; § Test Locally First, E2E). Spec UX : `_bmad-output/planning-artifacts/ux-design-specification.md`
l. 286, 446, 514, 639-662, 714-718.

## Le périmètre, tel que l'orchestrateur l'a arrêté en P2

Signal D5 levé en P2 (recyclage : la majorité des MEDIUM naissaient de la remédiation P1). Décision de
l'orchestrateur, sur l'autonomie déléguée par Guy : **pas de découpage, périmètre réduit au strict nécessaire pour
fermer #640** (C-28-1-26). La 28-1 se limite à : le jeton `--color-destructive` (+ `-foreground`, `-hover`) et ses
contrastes ; la variante `destructive` du `Button` ; les gardes Vitest (AC6 : table fermée de quatorze boutons
rouges, **sans migration**, et (f) ; AC7 (1) à (4)) ; la page `/design-system` ; une recette E2E centrée sur
*Supprimer l'import* et son dialogue, captures hors dépôt ; manuels, spec UX, CHANGELOG. **Retiré** : la migration
des cinq boutons stylés à la main, *Annuler le rapprochement* en `outline` (C-28-1-21 annulé), les scénarios (c),
(d), (e), les captures des 17 fichiers. **Hors périmètre, avec leur issue** : jetons non définis, déclencheurs
`ghost`, bordure `outline` → **#645** ; confirmation de suppression d'une personne de contact → **#646** ; les cinq
boutons stylés à la main → issue demandée à l'orchestrateur (angle mort 11).

Ne propose **pas** de réintégrer ce qui a été retiré : un finding qui revient à « il faut migrer les cinq sites »,
« il faut définir `muted` » ou « il faut une confirmation » n'est pas recevable. Est recevable tout ce que la fiche
**affirme** et qui serait faux, toute contradiction laissée par le retrait, et tout effet de la story qu'elle
n'aurait pas vu — en particulier sur les sites qu'elle ne touche plus.

## Axe premier — les régressions de la remédiation P2, et la cohérence du périmètre réduit

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire.* Lis d'abord le diff de la remédiation,
puis, pour chaque finding, **dis s'il est né de la remédiation P2** — et, si oui, s'il **recycle** un défaut déjà
signalé (même défaut sous une autre forme, ou né du correctif précédent) ou s'il est **neuf**. C'est la question
qui décide du découpage (CLAUDE.md, amendement D5) : réponds-y finding par finding.

1. **Cohérence du retrait (C-28-1-26)** : greppe toute trace du périmètre P1 — `AC5` au sens de migration,
   `sites migrés`, `M18`, `(c)`, `(d)`, `(e)`, `outline size`, `xs`, `C-28-1-21`, `C-28-1-24`, `sept modules`,
   `neuf entrées`, `douze`, `facultati`, `parseRgb`, `expect.poll`, `contact inventé`, `T10`, `M1-M20` — dans la
   fiche **et** le registre : chacune ne doit survivre que dans les Change Logs historiques, les entrées
   antérieures du registre suivies de leur rectification, ou une mention explicite de retrait. L'AC4, l'AC6,
   l'AC8, l'AC12, les classes B et D, les angles morts 2, 7, 8, 9, 11, le § *Fichiers probablement communs*
   disent-ils tous la même chose des cinq sites retirés ?
2. **AC5 / trois modules** : la liste des fichiers que la story édite dans `frontend/src/` (AC5, § *Découpage*)
   est-elle complète et vraie — l'AC7 (4) vit-elle bien sous `lib/components/ui/`, l'AC6 sous `ui/button/` ? Le
   décompte « trois modules » tient-il ? `frontend/package.json` et son lockfile (AC7 (4)) sont-ils bien classés ?
3. **AC6 (f) et la vérification par construction (C-28-1-28)** : « 24 usages, aucun avec `class` » est-il vrai
   (`grep`, et vérifie élément par élément ceux dont l'ouverture `<Button` est multiligne) ? (f) rougirait-elle au
   premier lancement ? La clé par `source.slice` de la valeur est-elle décidable dans l'AST de
   `svelte/compiler` 5.55 pour les quatorze entrées (lis `node_modules/svelte/types/index.d.ts`, positions
   `start`/`end` des nœuds `Text` et `ExpressionTag`) ? Les cinq clés écrites pour les classes B et D sont-elles
   exactement celles que rendrait cette définition ? Quatorze entrées : recompte-les (B 2, C 6, D 3, E 3).
4. **AC7 (4) / C-28-1-30** : `__unstable__loadDesignSystem(css, { base })` et `candidatesToCss` existent-ils avec
   ces signatures dans `@tailwindcss/node` 4.2.2 installé ? La frontière de jeton prescrite (séparateurs
   `` [\s"'`{}()] ``, préfixes jusqu'au dernier `:`, `!`, `/<n>`) produit-elle des candidats parasites ou en
   manque-t-elle (classes en gabarit, `class:` directives, chaînes de `tv(...)`, `[&_svg]:…`) ? La table de
   **treize** noms est-elle juste — refais le relevé indépendant par `grep` ? Le **repli** (lecture d'`app.css` et
   de `tailwindcss/theme.css`) est-il réalisable et dit-il honnêtement sa faiblesse ? M20 est-elle tuée dans les
   deux sens ?
5. **AC8 / C-28-1-29** : `expect.soft` + capture dans le `finally` d'un `test.step` laissent-ils bien douze
   captures en phase rouge (T0) — que se passe-t-il quand une action (survol, clic) lève dans un step, et quand un
   `toHaveCSS` soft expire ? Les deux passes en tests distincts paramétrés par `for … of` sont-elles valides
   Playwright (titres uniques) ? Le `beforeEach` `seedTestState('with-company')` dans (b) est-il compatible avec
   `bank-import-delete.spec.ts` et `workers: 1` ; le sélecteur structurel d'*Annuler*
   (`[data-slot="dialog-footer"] [data-slot="button"]:not([data-testid])`) vise-t-il un seul élément, sans
   libellé (`e2e-selecteurs-traduits.test.ts`) ? Le dialogue rendu dans un portail porte-t-il bien
   `data-testid="bank-import-delete-dialog"` sur l'élément qui contient le pied ?
6. **C-28-1-31** : sans `parseRgb` ni calcul en E2E, une mutation reste-t-elle sans test (M2, M5, M6 — vérifie le
   test qui tue chacune) ? Les cas « foncé en premier » de `contrast.test.ts` tuent-ils M16, et le cas blanc/noir
   M15 ? `blend('#b91c1c', '#ffffff', 0.1)` vaut-il `(248 ; 232,3 ; 232,3)` ?
7. **C-28-1-27, une seule méthode** : recalcule à la main (formule WCAG 2, sans interpréteur) au moins 5,46, 2,54,
   4,14 et 6,47, en montrant les calculs. Toute valeur citée dans la fiche, le registre (hors entrées historiques
   suivies de leur rectification), le commentaire d'`app.css` prescrit, l'AC11 et le CHANGELOG est-elle celle de
   cette méthode ?
8. **Le CHANGELOG (AC12)** dit-il exactement ce que la story livre, sans plus — *Annuler le rapprochement* « lien
   rouge », les cadres `danger` « sans couleur », les trois cadres `error-soft` « bordure rouge sans fond » :
   vrai au code ?

## Axes communs

1. **Inventaire clos** (Dev Notes, classes A à E, déclencheurs neutres bordés, déclencheurs `ghost`) : refais les
   commandes et élargis-les — *un détecteur part du symptôme, pas d'une liste*. Symptôme : *un contrôle qui
   déclenche une action destructive et ne se lit pas comme un bouton*. Un site manquant ou mal classé est au moins
   MEDIUM ; une erreur dans la table de l'AC6 (quatorze entrées) l'est aussi.
2. **Mesures** : voir l'axe premier 7 ; un ratio faux qui fait passer un seuil est HIGH.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` (l. 1107, 1245, 1725-1745, 2010,
   2016-2021) et `admin-manual.tex` (l. 803, 1172, 1985), et les **PDF** aplatis
   (`pdftotext <pdf> - | tr '\n' ' ' | tr -s ' '`). Ce que l'AC10 prescrit sera-t-il vrai au code prescrit ? Le
   manuel décrit-il *Annuler le rapprochement* ou un autre site retiré d'une façon que la story rend fausse (il
   devient rouge) ?
4. **Recomptes** : 24 / 17 / 201 / 42 / 32 / 73 / 29 / 11 / 19-16-3 / 16 dialogues (12 `outline`, 4 `ghost`) /
   14 entrées / 13 noms / 12 captures / 13 AC / T0-T9 / M1-M21 (M18 retirée) / C-28-1-1 à 31 / trois modules, et
   chaque numéro de ligne cité (`sed -n`, `grep -nF`).
5. **Cohérence interne** : une contradiction entre deux AC, entre un AC et le registre, ou entre la fiche et
   #645/#646/#640 (y compris ses commentaires, qui annoncent encore les « boutons stylés à la main ») est au moins
   MEDIUM.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase « Attendu » de #640 et du § 28-1 de l'epic couverte par un AC testable **ou** explicitement
  écartée avec son motif (C-28-1-26, C-28-1-28) ; chaque AC par une tâche et par un test qui rougirait ; chaque
  mutation M1-M21 par le test nommé ; chaque fait, chemin et numéro de ligne revérifié ; les textes prescrits
  (CHANGELOG AC12, manuels AC10, spec UX AC11) confrontés au code prescrit ; les gardes existantes nommées ; axes
  premier 1, 2, 3, 7, 8 en priorité.
- **F — Faisabilité et pièges techniques (adversaire de la conception)** : Tailwind **4.2.2**, tailwind-merge
  **3.5**, tailwind-variants **3.2**, `svelte/compiler` **5.55**, `@tailwindcss/node` **4.2.2**, Playwright
  (`toHaveCSS`, `expect.soft`, `test.step`, `emulateMedia`, focus clavier), tous en lecture seule sous
  `frontend/node_modules/`. Axes premier 3, 4, 5, 6 en priorité ; puis le déterminisme des deux scénarios et la
  survie des captures à un échec.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chacun : **né de la remédiation P2
(`git diff 3db1f345 f21d2db8`) ou de la conception d'origine**, et, s'il est né de la remédiation, **recyclage ou
défaut neuf** (axe premier). Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste
des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Bilan par sévérité
**recompté sur tes propres titres** avant d'être écrit. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-1-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus avec son origine (P2 recyclage / P2 neuf / origine).

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, `vite`, `vitest`, `playwright`, `docker`, `curl` vers
le backend, `bash` sur un script du dépôt, **tout interpréteur** (`python`, `python3`, `perl`, `ruby`, `node -e`,
`awk` à effet de bord), `gh` en écriture, toute requête SQL. Les questions de faisabilité se tranchent **par
lecture** du code des paquets installés, non par exécution : écris « non exécuté, lu à `fichier:ligne` ». Rien de
`~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `ls`, `wc`,
`sort`, `uniq`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
