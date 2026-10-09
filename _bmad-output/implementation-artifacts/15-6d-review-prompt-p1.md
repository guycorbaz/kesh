# Prompt — revue de code P1, Story 15-6d

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-6d`, branche `story/15-6d-contrepartie-distincte-de-la-banque`. **Le diff à revoir** :
`git -C /home/gcorbaz/devel/kesh-15-6d diff 803f3e15..cd0b581b -- . ':(exclude)_bmad-output'` (un seul diff aplati ; les
fiches `_bmad-output/` sont le contexte, pas l'objet). **Fiche** :
`_bmad-output/implementation-artifacts/15-6d-contrepartie-distincte-de-la-banque.md` (AC, tâches, T0, Dev Agent Record,
Change Log). Issue (par `gh api repos/guycorbaz/kesh/issues/524`) : #524 (P1). Registre : `epic-15-choix-autonomes.md`
(C-15-6-9, 23, 28, 31 ; C-15-6d-1). Règles : `CLAUDE.md`, dont § « Pattern batch — FailedProposal per-proposal ».

Code touché : refus d'une contrepartie égale au compte de la banque dans `post_manual` (400 `VALIDATION_ERROR`, ordre :
après le compte de banque actif, avant le 404, `ACCOUNT_NOT_POSTABLE` et l'état de la transaction) et dans
`accept_one_rule` (étape 1 bis : banque archivée → `BANK_ACCOUNT_NOT_CONFIGURED` ; étape 4 bis : `failed[]`
`VALIDATION_ERROR` / `counterparty_equals_bank_ledger`) ; comparaison par `invoice_settlements::ensure_not_claim_account`
(15-6b) ; deux gardes ventilées existantes ramenées aux fonctions locales ; `get_proposals` retire le compte de la banque
de `active_account_ids` ; écran `ReconciliationProposals` / `ManualMatchModal` (prop `bankLedgerAccountId`, lecture
unique de `listBankAccounts()`, repli `null` sans filtre) ; manuel, api-external, CHANGELOG (deux codes rendus avant
le correctif changent).

Axes : l'ensemble clos des chemins qui écrivent une contrepartie de rapprochement (manuel, règle, ventilé, lot,
`accept_batch`, rejeu, import) — un chemin échappe-t-il au refus ? ; l'ordre des refus sur les quatre chemins et la
réponse identique annoncée ; le pattern `FailedProposal` (aucune erreur par proposition qui escalade en `AppError`) ;
les codes qui changent pour les intégrateurs ; le choix C-15-6d-1 (doublure de la modale : le test 11 prouve-t-il le
câblage ?) ; le repli `null` (filtre silencieusement absent) ; le manuel (refus « sans réserve », filtrage « en règle
générale ») ; les angles morts déclarés (ventilation non filtrée, édition de règle, banque archivée avant
l'ouverture de l'écran).

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Défauts de correction, régressions, erreurs de concurrence
  (verrous, REPEATABLE READ, ordre d'acquisition), erreurs rendues au client, tests qui passeraient à vide (un test qui
  ne mord pas sur la mutation qu'il prétend couvrir), code mort, duplication (DRY), doc-comments devenus faux.
- **E — Edge Case Hunter** : chaque branche et chaque borne du code modifié — entrées vides, nulles, multiples, en
  doublon, d'une autre société, archivées ; chemins par clé d'API ; lots partiellement en échec ; locales ; et les
  **chemins NON modifiés qui devraient l'être** (inventorier les sites non résolus de la même famille par `grep`).
- **A — Acceptance Auditor** : chaque AC de la fiche contre le code ET les tests (un AC sans test qui le prouve est
  un finding) ; le Dev Agent Record ne déclare-t-il que ce qui a tourné (chiffres recomptés : `grep -c '#\[sqlx::test\]\|#\[test\]\|#\[tokio::test\]'` aux deux bornes) ;
  le **manuel** (`docs/manual/fr/*.tex` **et PDF aplatis** : `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' '` vers la sortie
  standard, ligatures ﬀ/ﬁ/ﬂ normalisées), `docs/api-external.md`, CHANGELOG, i18n 4 locales.

## Ce que tu rends

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-6d-review-p1-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni
aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
