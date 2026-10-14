# Prompt — validation P3 **ciblée** de la spec, Story 29-3 (la proposition affichée suit une priorité écrite)

*Versionné le 2026-10-14. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de la remédiation P2** (`CLAUDE.md`, § *La passe ciblée* : quand une boucle converge, ce qu'il reste à relire
n'est plus la story, c'est la dernière remédiation). Trend des MEDIUM distincts : P1 11 (dont 1 HIGH) → P2 3, tous
nés de la remédiation P1 sauf une partie de R3 ; 0 HIGH, 0 CRITICAL en P2.*

Dépôt `/home/gcorbaz/devel/kesh-29-3`, branche `story/29-3`. Le code au sol est celui de `16eeed21` : la fiche est
une spec, rien n'est encore implémenté.
**Périmètre : `git show b41e5b14`** (remédiation P2), soit un **diff unique, aplati** — ne lis pas la séquence des
commits antérieurs. Fichiers touchés : la fiche `_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`,
le registre `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-3.md` (C-29-3-7, 8, 11, 12, 15 annotés ;
C-29-3-16 à 18 ajoutés ; en-tête), `sprint-status.yaml` (ligne `29-3-…`). Rapports P2 que ce commit remédie :
`/home/gcorbaz/devel/kesh-gate-logs/29-3-validate-p2-{R,F}.md`. Fiche voisine, **en lecture seule** :
`/home/gcorbaz/devel/kesh-29-2/_bmad-output/implementation-artifacts/29-2-paiement-tardif.md` et son registre
`epic-29-choix-autonomes-29-2.md` (même dossier) — elle se corrige de son côté, lis-la telle qu'elle est.

## La lentille — chasseur de régressions de la remédiation P2

Pour **chaque hunk** du commit, vérifie contre le code au sol :

1. **Phrase de l'écran, AC5 (R1, C-29-3-15 annoté)** : « Une facture proposée passe toujours devant une règle. »
   est-elle vraie sur **toute** sortie de `get_proposals` (`crates/kesh-api/src/routes/reconciliation.rs:722-827` :
   la règle est-elle toujours poussée **après** les factures, et seulement si aucune n'atteint 0,5 ?) ? Les versions
   de-CH, it-CH, en-CH disent-elles la même chose, ni plus ni moins ? La justification ajoutée (candidature par le
   reste dû à 5 centimes et la fenêtre, `crates/kesh-db/src/repositories/reconciliation.rs:250-259`) est-elle exacte ?
   Reste-t-il une formulation de condition suffisante **active** (`grep -nE "dès que|sobald|non appena|as soon as"`
   sur la fiche et le registre) ailleurs que dans une citation datée ou un texte d'origine suivi d'une annotation ?
   La Story (en tête de fiche) est-elle cohérente avec la phrase ?
2. **Trop-perçu #665 (R2/F1, C-29-3-16)** : la fiche affirme que le règlement depuis la fiche refuse le montant
   reçu — vrai à `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:179-184`, à
   `crates/kesh-db/src/repositories/invoice_settlements_write.rs:217-228` et à `docs/manual/fr/user-manual.tex:1308` ?
   Plus **aucun** site de la fiche ou du registre ne prescrit-il un contournement pour #665
   (`grep -nF "enregistrez alors le règlement"`) hors citation datée ? Le contrôle prescrit
   `grep -c "depuis la fiche de la facture (issue"` rend-il **0 au sol** sur le PDF aplati
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) — sinon il ne prouverait rien ?
   L'écartée de C-29-3-16 (« saisir le reste dû, puis rejeter ; l'écart n'est pas comptabilisé ») est-elle exacte au
   code ?
3. **Départage #668, angle mort C-29-3-12, C-29-3-17 (R3/F4)** : #668 existe-t-il et son second commentaire
   porte-t-il l'attendu cité (art. 87 CO : échues d'abord, puis la plus ancienne) (`gh api
   repos/guycorbaz/kesh/issues/668/comments`) ? La portée de l'angle mort (échues, relancées ou pas encore échues,
   échéance lointaine sans limite de temps) reproduit-elle C-29-2-7 et C-29-2-11 **tels qu'ils sont** au registre
   de la 29-2 ? `ORDER BY i.date ASC, i.id ASC` et `LIMIT 50` sont-ils à `reconciliation.rs:258-259` du dépôt ?
   « Au-delà de 50 candidates de même reste dû, ils écartent les plus récentes » est-il exact ? Le départage de
   `propose_matches` est-il à `crates/kesh-reconciliation/src/matching.rs:97-105` ? Reste-t-il « issue demandée » ou
   « ouvrira » dans une partie **vivante** de la fiche (`grep -nE "issue demandée|ouvrira"`) ?
4. **Tableau des zones partagées (R3)** : chaque ligne de la colonne « zone 29-2 » est-elle tirée de la fiche 29-2
   (§ *Modules touchés*, § *Zones partagées avec la 29-3*, § *Exclusions écrites*, C-29-2-5, C-29-2-6) ? En
   particulier : la 29-2 ne touche-t-elle vraiment ni frontend ni i18n ; touche-t-elle `accept_one_invoice` et le
   commentaire de la passe 2 aux lignes citées ; ajoute-t-elle une phrase au même paragraphe de
   `docs/api-external.md` (`:547-567`) ; ses sites du manuel sont-ils `:1305`, `:2029`, `:2031`, `:2902` ? Le
   paragraphe « Ce que la fiche 29-2 attend encore de la 29-3 » cite-t-il sa version actuelle sans la déformer ?
5. **Refus d'une facture proposée (F9/R9)** : les codes cités (`FISCAL_YEAR_INVALID`, `PERIOD_LOCKED`,
   `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, `RECONCILIATION_ALREADY_RECONCILED`, `INVOICE_SALE_ENTRY_MALFORMED`)
   sont-ils atteignables depuis `accept_one_invoice` (`reconciliation.rs:1621-2330`, et les helpers qu'il appelle) ?
   Leur classement (état comptable, course, intégrité) est-il juste ? `user-manual.tex:2056` et `:2060` disent-ils
   ce que la fiche leur fait dire ?
6. **Mutations M7, M11 et la variante de M1 (F2/R4, F5)** : recalcule, contre les onze cas de l'AC2 et les deux tests
   HTTP de l'AC3, la liste « doit faire rougir » de M7 (cas 1, 2, 3, 4, 6, 11) et de M11 (cas 5, 6, 10, 11) — une
   liste est fausse si un cas déclaré reste vert **ou** si un cas non déclaré rougit. La variante « appel placé
   avant le `push` de la règle » est-elle bien équivalente sur les entrées atteignables ? La note de recalcul sous
   le tableau est-elle exacte ?
7. **E2E, AC6 et C-29-3-18 (R8)** : le gabarit `frontend/tests/e2e/fixtures/camt053_v04_minimal.xml` porte-t-il bien
   `<Ref>`, `<Ustrd>`, `<EndToEndId>`, `<AcctSvcrRef>` ? Lequel alimente la référence de la transaction à l'import
   (`crates/kesh-import/src/camt053/`), et le remplacement prescrit suffit-il à rendre le `reference_score` nul pour
   toute autre facture ? `POST /api/v1/invoices/{id}/settlements` existe-t-il (route au serveur), et un règlement de
   972,89 sur 972,90 laisse-t-il 0,01 (`classify_payment`, `invoice_settlements.rs:175-184`) ? « Toutes les factures
   de `createAndValidateInvoiceViaApi` ont ce TTC » : vrai à `frontend/tests/e2e/helpers/api-fixtures.ts:100-127` ?
8. **API externe, AC8 (F8)** : la règle porte-t-elle des sous-scores nuls (`reconciliation.rs:818-823`) ? Le JSON
   sérialise-t-il `candidateType` en `"rule"` / `"invoice"` (attributs `serde` de `CandidateType`, `:119`) ?
9. **Rang 2 et risque résiduel (R6, R7)** : la décision L17 est-elle à `matching.rs:112-114` ? #666 existe-t-il et
   porte-t-il le choix manuel ? Les trois voies du risque résiduel correspondent-elles à `matching.rs:132`, `:142`,
   `:158` ? « Un poids changé, resté positif, ne peuple pas ce rang » est-il exact ?
10. **Recomptes et historique** : 18 titres `### C-29-3-` au registre ; 11 lignes `| M<n> |` ; 9 AC ; 8 tâches ;
    « quatre ajouts » au manuel (comptés au tableau de l'AC7) ; trend « R 0/0/3/6, F 0/0/1/8, 18 findings, 13 sujets
    distincts, 3 MEDIUM distincts » contre les sections *Bilan* et *Findings* des deux rapports P2 —
    **recompte depuis les fichiers, ne relis pas les nombres.** Le commit a-t-il réécrit une entrée **passée** (le
    Change Log de la création et de la P1, la partie P1 du commentaire de `sprint-status.yaml`, le texte d'origine
    d'un choix du registre) au lieu d'annoter ? `git show b41e5b14 -- <fichier> | grep "^-"` le dit.

⚠️ **Erreur de catégorie** (consigne 5 des agents) : un livrable **prescrit par la fiche mais absent du code** n'est
**pas** un défaut — la fiche est une spec, le code n'est pas encore écrit (« `candidate_rank` n'existe pas »,
« le manuel ne contient pas l'ajout », « la clé `reconciliation-labels-priority-hint` est absente des catalogues »,
« M11 n'a pas été jouée » sont **attendus**). Un finding ne porte que sur ce que **le texte du commit affirme du code
existant**, ou sur ce qu'il **prescrit de faux ou d'infaisable**.
⚠️ Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée (`-F` obligatoire sur les motifs à
métacaractères). Un CRITICAL ou HIGH qui affirme qu'un patch « n'a pas été appliqué » se vérifie au `grep -nF` sur
le fichier **final** (pas sur une ligne du diff) avant d'être écrit.
⚠️ Données réelles : rien de `~/kesh-test/` (relevé réel de Guy), ni dans le rapport ni dans une commande.

## Ce que tu rends

Findings numérotés `P3-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), origine (remédiation P2 ou conception d'origine), correction proposée. ⛔ **La liste des axes
exercés ET non exercés** (les dix ci-dessus, un par un, avec ce qui a été fait ou pourquoi pas) — un « 0 finding »
sans elle ne compte pas comme passe, et un axe déclaré exercé sans commande copiée compte comme non exercé.
Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-3-validate-p3-ciblee.md` ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/29-3-validate-p3-ciblee.md`. Aucune commande qui
écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`restore`/`stash`/
`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, sous **aucune** forme et dans **aucun** worktree, `cargo`,
`npm`, `npx`, `node`, `docker`, `mysql`/`mariadb` et toute requête SQL, `gh` en écriture, `sed -i`, et **tout
interpréteur** (`python`, `python3`, `perl`, `ruby`, `node`, `deno`, `bash`/`sh` sur un script ou avec `-c`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `wc`, `git log`/`show`/`diff`/`blame`, `gh api` et
`gh issue view` en lecture, `pdftotext` vers la sortie standard.
