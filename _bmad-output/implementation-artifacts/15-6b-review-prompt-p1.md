# Prompt — revue de code P1, Story 15-6b

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-6b`, branche `story/15-6b-contrepartie-distincte-de-la-creance`. **Le diff à revoir** : `git -C /home/gcorbaz/devel/kesh-15-6b diff 39b52628..1b006a89` (un seul diff aplati ; le code,
les tests, les manuels, la doc — les fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-6b-contrepartie-distincte-de-la-creance.md` (AC, tâches, Dev Agent Record, Change Log). Issues (par
`gh api repos/guycorbaz/kesh/issues/N`) : #474. Registre : `epic-15-choix-autonomes.md`. Règles : `CLAUDE.md`. Choix C-15-6-*, C-15-6b-1, C-15-6b-2. Code touché : variante SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT (400), helper refuse_if_claim_account, gardes sur règlement client, solde du reste, règlement fournisseur (purchase_payable_line), création et confirmation des lots, rapprochement ; GapAccountRole ; écrans (account-options.ts, dialogue de règlement, fiche fournisseur, libellés failed[]) ; i18n 4 locales ; manuels et PDF, README, api-external, CHANGELOG. Axes : l'ensemble clos des chemins qui écrivent une contrepartie de règlement — en manque-t-il un (inventaire des sites NON gardés, `grep -rn` sur les écrivains de `invoice_settlements` et de règlements fournisseur) ? ; le refus dans les lots suit-il le patron FailedProposal per-proposal (CLAUDE.md) ? ; ordre du refus parmi les autres refus de ces flux ; lecture sous verrou ou non ; la doublure SettleInvoiceDialogHost.test.svelte ne masque-t-elle pas un défaut du composant ? ; la borne CLES_RELEVEES 210→213 ; manuel et PDF aplati.

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Défauts de correction, régressions, erreurs de concurrence
  (verrous, REPEATABLE READ, ordre d'acquisition), erreurs rendues au client, tests qui passeraient à vide (un test qui
  ne mord pas sur la mutation qu'il prétend couvrir), code mort, duplication (DRY), doc-comments devenus faux.
- **E — Edge Case Hunter** : chaque branche et chaque borne du code modifié — entrées vides, nulles, multiples, en
  doublon, d'une autre société, archivées ; chemins par clé d'API ; lots partiellement en échec ; locales ; et les
  **chemins NON modifiés qui devraient l'être** (inventorier les sites non résolus de la même famille par `grep`).
- **A — Acceptance Auditor** : chaque AC de la fiche contre le code ET les tests (un AC sans test qui le prouve est
  un finding) ; le Dev Agent Record ne déclare-t-il que ce qui a tourné (chiffres recomptés : `grep -c '#\[sqlx::test\]\|#\[test\]\|#\[tokio::test\]'` aux deux bornes) ;
  le **manuel** (`docs/manual/fr/*.tex` **et PDF aplatis** : `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' '` vers
  `target/gate-logs/`, ligatures ﬀ/ﬁ/ﬂ normalisées), `docs/api-external.md`, CHANGELOG, i18n 4 locales.

## Ce que tu rends

Rapport complet dans `target/gate-logs/15-6b-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers `target/gate-logs/`.
