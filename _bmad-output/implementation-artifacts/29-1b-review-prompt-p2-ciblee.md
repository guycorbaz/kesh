# Prompt — passe ciblée P2, revue de code de la Story 29-1b (une lentille, la seule remédiation P1)

*Versionné le 2026-10-14. Une lentille **Sonnet**, contexte frais, **lecture seule stricte**. Passe ciblée au sens de
`CLAUDE.md` § « La passe ciblée — une seule lentille, braquée sur la dernière remédiation » : ce qui reste à relire
n'est plus la story, c'est la remédiation de la revue P1.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1b`.

**Le diff à revoir, et lui seul** : `git diff 76129d3d 49ad0020 -- . ':(exclude)_bmad-output'` — deux commits :
`05f44ecc` (écran, catalogues ×4, Vitest, E2E, doc-comments Rust — **dernier commit de code**) et `49ad0020` (manuel et
brochure avec leurs PDF, README, `website/roadmap.html`). ⚠️ Pathspec `':(exclude)…'`, **non** `':!…'`. Lis le diff en
une fois (aplati), pas commit par commit. Le reste de la story a été revu en P1 : n'y reviens que pour juger un site
que la remédiation touche ou aurait dû toucher.

Contexte à lire : les trois rapports P1 `/home/gcorbaz/devel/kesh-gate-logs/29-1b-review-p1-{B,E,A}.md` ; la fiche
`_bmad-output/implementation-artifacts/29-1-preparer-cloture.md`, Dev Agent Record § *29-1b — remédiation de la revue
de code P1* (décisions de l'orchestrateur, findings traités, mutations M-b14 à M-b22, gates) ; le registre
`epic-29-choix-autonomes-29-1.md`, choix **C-29-1-59 à 64**. Journal de mutations :
`/home/gcorbaz/devel/kesh-gate-logs/291b-p1-mutations.log` ; gates : `291b-p1-final-*.log` au même endroit.

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de `/home/gcorbaz/devel/kesh-gate-logs/`.

## La lentille — régression de la remédiation

La question n'est pas « la story est-elle juste ? » mais : **le patch P1 a-t-il introduit un défaut, ou n'a-t-il réglé
qu'une copie du symptôme ?** Axes, chacun à déclarer exercé ou non :

1. **`loadEntry` ferme les confirmations** (`+page.svelte`, B-1, C-29-1-60). Chaque appelant de `loadEntry`
   (`$effect`, `onEditSaved`, `onEditStale`, refus de suppression, refus du geste neuf) : une confirmation **devait-elle**
   rester ouverte sur l'un d'eux ? Le refus du geste **du jour** ne relit pas la fiche et garde son dialogue ouvert :
   toujours vrai ? Un `await` qui précéderait la remise à zéro (fenêtre où le dialogue survivrait) ? M-b18 déclarée
   équivalente : l'argument (remise à zéro synchrone avant le premier `await` de `loadEntry`) tient-il ?
2. **La confirmation nomme l'écriture** (C-29-1-59) : `{ $number }`, `{ $description }` dans les **quatre** catalogues
   et le repli Svelte (égal à fr-CH, caractère pour caractère), les variables de chaque locale identiques à celles du
   repli ; un libellé vide, très long, ou contenant `{ $x }` / des guillemets casse-t-il l'interpolation de `i18nMsg`
   (`frontend/src/lib/shared/utils/i18n.svelte.ts`) ? La traduction allemande, italienne, anglaise est-elle juste ? Le
   texte cité au manuel et attendu par l'E2E est-il celui que l'écran rend ?
3. **Les tests neufs prouvent-ils ce qu'ils nomment** (`journal-entry-page.test.ts`, bloc « revue de code P1 », et le
   module `page-state.test-helper.svelte.ts`, C-29-1-61). Le `page` réactif fait-il **réellement** rejouer le `$effect`
   (les cas attendent `Écriture n°10`) ; `afterEach` le remet-il à `7` pour les cas suivants ? Un cas qui passerait à
   vide : `queryByRole('dialog')` nul pour une autre raison que la fermeture, un `setTimeout(0)` trop court pour la
   réponse tardive, un mock `mockReturnValueOnce` consommé par un autre appel que celui visé. Relis le journal
   `291b-p1-mutations.log` : chaque mutation tuée l'est-elle par le cas qui la revendique ? Cherche une **mutation
   plausible qui resterait verte** dans le code touché par le patch.
4. **L'E2E de parcours** (`fiscal-year-closing-journey.spec.ts`) : le numéro d'écriture lu par l'API avant l'assertion
   du texte ; `details.accounts[0].count` est-il bien le champ servi par K2 (`crates/kesh-db/src/repositories/
   closing_checks.rs`, type `PendingByAccount` de `frontend/src/lib/features/fiscal-years/fiscal-years.types.ts`) ;
   `toHaveText('Aucun point à vérifier.')` contre le repli **et** le catalogue fr-CH servi.
5. **Les doc-comments Rust** (`crates/kesh-db/src/errors.rs`, `journal_entries.rs` : B-5, E-4, E-5) disent-ils vrai au
   code — notamment « aucun chemin ne fait disparaître l'exercice suivant » (`grep -rn "DELETE FROM fiscal_years"
   crates/kesh-db/src`) et « davantage de lignes quand le geste est fait avant le début de l'exercice suivant » ?
   Vérifie qu'**aucune ligne exécutable** de Rust n'a changé (`git diff 76129d3d 05f44ecc -- crates/*/src` : seuls des
   `///` / `//`).
6. ⛔ **Documentation — MANUEL ET BROCHURE (PDF compris), README, site.** Chaque phrase neuve dit-elle vrai au code ?
   Manuel `docs/manual/fr/user-manual.tex`, § *Les transitoires de fin d'exercice* : la confirmation citée ; « ce motif
   s'affiche sous toute écriture contre-passable tant que l'exercice suivant n'existe pas » ; « le bouton reste offert…
   un refus pour ce motif se répète » (vrai pour `PERIOD_LOCKED` — et pour `LATER_FISCAL_YEAR_CLOSED` ?) ; « créé et
   ouvert ». Brochure `marketing-brochure.tex` : le bloc Epic 29 « en cours — à paraître » (v0.13.4 pour les contrôles,
   v0.14.0 pour les transitoires — conforme à C-29-1-43 ?), plus rien de l'Epic 29 sous « Epic 14, v0.8, livré ».
   README : la ligne E29 « 🚧 En cours » et les marques « à paraître » ne promettent-elles rien de publié ? Les issues
   citées (#529, #527, #668) disent-elles ce que la ligne leur prête (`gh issue view <N>` en lecture) ? Garde G7 (tout
   `#N` du README est un lien vers l'issue N). Site `website/roadmap.html` : la phrase de l'en-tête et la carte E29.
   **PDF aplatis, en pipe** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "…"`
   (idem `marketing-brochure.pdf`) — ⚠️ apostrophe typographique « ’ », ligatures, coupures de mots en fin de ligne :
   un `grep` naïf rend un faux négatif. Les PDF sont-ils ceux du `.tex` du HEAD (`git log -1 --format=%h -- <f>` des
   quatre fichiers) ?
7. **Propagation du symptôme** : greper la **valeur** de chaque correction sur tout le worktree — l'ancien texte de la
   confirmation (`grep -rnF "La contre-passation sera datée du"`), « a disparu » / « supprimé entre-temps » à propos de
   l'exercice suivant, « d'un clic » / « d’un clic » à propos des transitoires, « year-end closing entries »,
   « Années fixes et passées ». Un reste non traité est un finding.
8. **Comptes rendus** : recompte depuis la source les décomptes de l'entrée « revue P1 » du Dev Agent Record et du
   Change Log (cas Vitest neufs aux **deux** bornes `76129d3d` / `05f44ecc`, mutations exécutées / tuées /
   équivalentes, gates contre `291b-p1-final-*.log`) et le **périmètre** déclaré de chaque nombre ; aucun `closes` /
   `fixes` dans un message de commit (`git log 76129d3d..HEAD --format=%B | grep -niE "close[sd]? #|fix(e[sd])? #"`).

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-1b-review-p2-ciblee.md`, et **aucun autre fichier**. Findings
avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée, ou code relu),
correction proposée ; pour toute affirmation de présence ou d'absence, la sortie d'un `grep -nF` copiée. Précise pour
chaque finding s'il **naît de la remédiation P1** ou s'il lui préexistait. ⛔ **La liste des axes exercés ET non
exercés** (les huit ci-dessus) — un « 0 finding » sans elle ne compte pas.

## Interdits — lecture seule stricte

⛔ **Aucun fichier écrit** hors de ton rapport, **pas même un fichier temporaire** : les PDF s'aplatissent **en pipe**
(`pdftotext … - | tr … | grep`), jamais vers un fichier ; toute comparaison se fait **en pipe** (`diff <(git show A:f)
<(git show B:f)`, `cmp <(…) <(…)`), sans redirection `>` ni `tee`. Aucune commande qui écrit dans le dépôt ou dans une
base, ni qui compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/
`rebase`/`restore`/`push`/`tag` — **aucun `git checkout` ni `git switch` sous aucune forme**, pas même `--detach` —,
`sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`, `node`, `deno`, `bun`, **`python`/`python3`** (pas même le script
de mutations), `perl`, `ruby`, `php`, **`sed -i`**, `awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`,
`docker`, `curl`, `wget`, `gh` en écriture, `pkill`/`kill`, aucune requête SQL. Autorisés : lecture, `grep`, `sed -n`,
`wc`, `sort`, `cmp`, `diff` en pipe, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers la sortie standard.
