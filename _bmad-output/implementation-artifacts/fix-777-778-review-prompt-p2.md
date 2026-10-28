# Revue de code P2 — issues #777 et #778 : passe ciblée « regression hunter » sur la remédiation de la P1

Prompt versionné. **Passe ciblée** : une seule lentille, braquée sur le **seul commit de remédiation** de la P1.
Modèle : **Opus** (à défaut Sonnet — jamais Haiku pour cette passe), contexte frais, lecture seule.

## Objet

Worktree : `/home/gcorbaz/devel/kesh-777`, branche `fix/777-778-grand-livre-exercices`.
**Commit à relire : `772f7018`** — `git -C /home/gcorbaz/devel/kesh-777 show 772f7018`.
Contexte (à lire, non à revoir) : le correctif d'origine `976d31f0`, le prompt de la P1
(`fix-777-778-review-prompt-p1.md`, même dossier) et ses trois rapports
`/home/gcorbaz/devel/kesh-gate-logs/777-review-p1-{B,E,A}.md`.

La P1 a rendu 0 CRITICAL, 0 HIGH, 3 MEDIUM (un seul défaut, vu par les trois lentilles) et 10 LOW. Le commit
`772f7018` remédie :

1. **MEDIUM P1-B-1 = P1-E-1 = P1-A-1** — à l'écran (`GeneralLedgerView.svelte`, branche `{:else}` du
   `{#each s.lines}`), « Aucun mouvement sur la période. Le solde d'ouverture reste dû. » était faux quand une
   rupture finale clôt l'ouverture (compte de résultat sans ligne, période qui franchit une fin d'exercice :
   clôture 0). Remède : si `trailingBreaks(s)` est non vide, nouvelle clé `reports-ledger-no-movement-closed`
   (« … L'exercice de l'ouverture se clôt dans la période : le solde repart de zéro. »), dans les quatre locales
   `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` ; test Vitest (P1-A-3). **Décision du développeur sur
   le PDF** : non touché — `crates/kesh-report/src/pdf.rs` écrit `ctx.section_labels.ledger_no_movement`, dont la
   valeur (l. ~203) est « Aucun mouvement sur la période. », sans « reste dû », et que `kesh-api` ne surcharge
   nulle part. **À vérifier**, la P1 était partagée (B disait touché, E non).
2. **LOW P1-E-2, P1-B-2 = P1-E-3** — `docs/api-external.md` : une rupture par changement d'exercice, aucune pour
   un exercice intermédiaire sans ligne ; l'identité `closing = opening + mouvements − Σ closingBalance` ne vaut
   que sur une lecture entière.
3. **LOW P1-B-3** — doc-comment de module de `crates/kesh-report/src/general_ledger.rs` (l. ~31-37).
4. **LOW P1-A-2 = P1-E-4** — description de l'outil MCP `kesh_grand_livre`
   (`crates/kesh-api/src/mcp/outils/grand_livre.rs`, `fn description`) : définit `cloture` et `ruptures` ; test
   unitaire `la_description_definit_cloture_et_ruptures`.
5. Garde `frontend/src/lib/shared/i18n-keys.test.ts` : `sitesTotal` 2393 → 2394.

Restent hors périmètre, tracés en issue par l'orchestrateur : P1-E-5 (page vide à `offset == line_count`), la
double sémantique de `FiscalYearBreak.date`, le test PDF sur la seule taille. Ne les rapporte pas de nouveau.

Journal des mutations : `/home/gcorbaz/devel/kesh-gate-logs/777-mutations.log` (M3 écran, M4 description MCP).

## Lentille — Regression Hunter

Le motif à chasser : *la sévérité se déplace vers ce qu'on vient d'écrire.* Cherche ce que **ce commit** a cassé
ou laissé faux, pas les défauts d'origine.

Axes, au minimum :

- **R1 — condition de l'écran.** `trailingBreaks(s).length > 0` dans la branche `{:else}` (aucune ligne rendue) :
  vrai exactement quand une rupture finale clôt l'ouverture ? Cas : compte de bilan sans ligne (aucune rupture) ;
  compte de résultat sans ligne dans un seul exercice ; écran tronqué ou page au-delà de la fin (`lines` vide mais
  `lineCount > 0` : quelles ruptures le backend rend-il alors, et quel message s'affiche ?) ; ouverture nulle
  (le compte est-il même rendu ?). Le nouveau texte est-il vrai dans chacun de ces cas ?
- **R2 — les quatre locales.** La clé neuve existe-t-elle dans les quatre `messages.ftl`, à côté de sa sœur, avec
  un sens fidèle au français ? Syntaxe Fluent (le `:` en milieu de valeur, l'apostrophe) sans piège ?
  `grep -nF "reports-ledger-no-movement-closed"` sur les quatre fichiers et sur `frontend/src`.
- **R3 — le PDF.** Tranche la question 1 à la lecture du code, lignes citées : le PDF affiche-t-il une phrase
  fausse dans ce cas ? (`grep -nF "ledger_no_movement"` sur `crates/`.)
- **R4 — les gardes.** Le relèvement de `sitesTotal` est-il juste (recompte `grep -oE '(i18nMsg|[^a-zA-Z]msg)\('`
  sur `GeneralLedgerView.svelte` aux deux bornes `782bbc2f` et `772f7018`) ? Une autre garde (i18n, lexicales MCP
  `crates/kesh-api/tests/mcp_gardes_lexicales.rs`, matrice du manuel `mcp_matrice_manuel.rs`, descripteurs figés de
  `mcp_transport_e2e.rs`) est-elle concernée par la nouvelle description MCP ou la nouvelle clé ?
- **R5 — la description MCP.** Dit-elle vrai au code (`general_ledger.rs`, `closing`) — en particulier pour un
  compte de bilan et pour une période d'un seul exercice, où `cloture` *est* l'ouverture plus les totaux ? Une
  phrase qui sur-promet est un défaut. Longueur raisonnable pour un descripteur `tools/list` ?
- **R6 — `docs/api-external.md` et le doc-comment.** Les nouvelles phrases disent-elles vrai au code (`Cumul::avance`,
  `Cumul::termine`, `page_finale`) ? La rupture de **tête** (#777) est-elle toujours couverte par la formulation
  « à chaque changement d'exercice entre deux lignes » et la phrase qui suit ?
- **R7 — les tests ajoutés.** Le test Vitest et le test unitaire MCP peuvent-ils passer à vide ? Les mutations M3
  et M4 du journal sont-elles les bonnes, tuées par le test visé ?
- **R8 — propagation.** Le symptôme (« reste dû », « remains outstanding », « bleibt offen », « resta dovuto »)
  subsiste-t-il ailleurs : manuels `docs/manual/fr/*.tex` et leur PDF aplati
  (`pdftotext /home/gcorbaz/devel/kesh-777/docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), CHANGELOG ?

## ⛔ Interdits

- **N'exécute JAMAIS `scripts/prepare-release.sh`**, ni directement ni par aucun détour. N'exécute aucun script du
  dépôt, ni `cargo`, `npm`, `npx`, `sqlx`, `docker`, `mariadb`/`mysql`, `make`/`latexmk`.
- **Aucune redirection** (`>`, `>>`, `<`, `tee`, here-doc) : compare par pipe, lis vers la sortie standard.
- **Ni `rm` sous aucune forme, ni `bc`.**
- N'écris, ne modifie, ne crée aucun fichier — **sauf ton rapport**, écrit avec l'outil d'écriture de fichiers.
  Aucune commande git ou `gh` qui écrit. Aucun `pkill`/`kill`.
- N'ouvre jamais `~/kesh-test/`.
- Commandes autorisées : `git show`, `git diff`, `git log`, `grep`, `sed -n`, `cat`, `pdftotext` (vers `-`), `tr`,
  lecture de fichiers.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/777-review-p2-R.md` :

1. **Axes exercés** et **axes NON exercés** — liste explicite, R1 à R8. Un « 0 finding » sans cette liste ne compte
   pas.
2. Findings numérotés `P2-R-n`, avec sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne, description,
   preuve, correctif proposé. **Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF`
   copiée** ; toute affirmation sur le manuel, sur le PDF aplati. Distingue ce que `772f7018` introduit de ce qui
   lui préexiste.
3. Verdict : nombre de findings par sévérité, et si la remédiation touche ou non du code de production.
