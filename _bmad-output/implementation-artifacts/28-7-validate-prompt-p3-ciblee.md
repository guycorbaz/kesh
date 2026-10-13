# Prompt — validation P3 **ciblée** de la spec, Story 28-7 (une facture postérieure au paiement n'est plus proposée)

*Versionné le 2026-10-13. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **le seul
commit de la remédiation P2** (`CLAUDE.md`, § *La passe ciblée* : quand une boucle converge, ce qu'il reste à relire
n'est plus la story, c'est la dernière remédiation). Trend des MEDIUM distincts : P1 4 → P2 2, 0 HIGH, 0 CRITICAL.*

Dépôt `/home/gcorbaz/devel/kesh-28-7`, branche `story/28-7`. Le code au sol est celui de `5aad8b35` : la fiche est
une spec, rien n'est encore implémenté.
**Périmètre : `git show bbace781`** (remédiation P2), soit un **diff unique, aplati** — ne lis pas la séquence des
commits antérieurs. Fichiers touchés : la fiche `_bmad-output/implementation-artifacts/28-7-facture-posterieure-paiement.md`,
le registre `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` (C-28-7-4 annoté, C-28-7-10 reformulé,
C-28-7-11 à 13 ajoutés), `sprint-status.yaml` (ligne `28-7-…`). Rapports P2 que ce commit remédie :
`/home/gcorbaz/devel/kesh-gate-logs/28-7-validate-p2-{R,F}.md`.

## La lentille — chasseur de régressions de la remédiation P2

Pour **chaque hunk** du commit, vérifie contre le code au sol :

1. **AC8, puce `:1921` (F1, C-28-7-11)** : la phrase prescrite sur le règlement depuis la fiche (« l'écran refuse
   toute date antérieure à la date de facture et n'en propose aucune au sélecteur ») est-elle vraie à
   `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:155` et `:295`, et à `WriteOffDialog.svelte:105`
   et `:211` ? « le serveur en admet une, que l'écran ne laisse pas atteindre » : vrai à
   `crates/kesh-db/src/repositories/invoice_settlements_write.rs:115` et `:489` ? Le serveur est-il atteint par un
   autre chemin d'écran que ces deux dialogues (greper `settle`/`write-off` dans `frontend/src`) — si oui, la phrase
   « que l'écran ne laisse pas atteindre » est-elle encore juste ?
2. **AC8, titre (R1)** : « Une facture datée de plus d'un jour après le paiement n'est pas proposée » est-il vrai à
   la règle de l'AC1 (`D − 30 ≤ F ≤ D + 1`), au décalage `+1` de l'AC4 et au cas (d) de l'AC5 ? Le corps prescrit
   et l'AC8 `:2787` disent-ils la même chose ?
3. **Inventaire (b) et AC9 (F2, F4 = R2)** : les sorties de grep ajoutées sont-elles **exactement** celles du sol
   (refais `grep -nF 'settledOn < invoiceDate' frontend/src/lib/features/invoices/*.svelte` et
   `grep -nF 'min={invoiceDate}' frontend/src/lib/features/invoices/*.svelte`) ? Reste-t-il, dans la fiche ou le
   registre, une phrase qui lie la CHECK à un bump de `min_required` (`grep -nE "bump|P2/P3|sans migration"`) ?
   L'inventaire (a) (« le frontend ne choisit aucune facture pour une transaction ») est-il vrai ?
4. **Angle mort CAMT (F3, C-28-7-12, C-28-7-4 annoté)** : `crates/kesh-import/src/camt053/mod.rs`, fonction
   `parse_date`, accepte-t-elle toute année ? `crates/kesh-api/src/routes/bank_imports.rs` borne-t-il l'année
   (refais `grep -rn "9999\|year()\|MIN_DATE\|MAX_DATE\|out of range\|hors plage" crates/kesh-import/src
   crates/kesh-api/src/routes/bank_imports.rs`) ? Le précédent `crates/kesh-api/src/routes/letterings.rs:507-511`
   dit-il ce que la fiche lui fait dire, ni plus ni moins ? Une autre phrase affirme-t-elle encore que le cas est
   inatteignable (`grep -nF "aucun chemin"`) ailleurs que dans le texte barré de C-28-7-4 ?
5. **Ordre de l'index et M10 (F5 = R3 = R4)** : les noms `idx_invoices_company_validated_unpaid_date` et
   `idx_invoices_company_date` existent-ils dans `crates/kesh-db/test-schema/0001_schema_squash.sql` ? Le tri aval
   `score DESC, invoice_id ASC` est-il à `crates/kesh-reconciliation/src/matching.rs:104-110` ? « Aucun test ne lit
   l'ordre du SQL » : refais `grep -nE "candidates(_a|_b)?\[[0-9]\]|found_[a-z]+\[[0-9]\]|\.first\(\)"
   crates/kesh-db/tests/reconciliation_repository.rs crates/kesh-db/tests/invoice_write_off.rs`. L'en-tête de l'AC10
   et la ligne M10 se contredisent-ils encore ? La note de correspondance des numéros (AC10, Change Log P2)
   est-elle cohérente avec le Change Log P1 (« l'ancienne M10 retirée, le numéro réattribué ») ?
6. **Mutation M15 et assertion R5** : M15 (`<` → `<=` sur `earliest` dans `check`) est-elle tuée par l'AC6
   (`D − 30 → Ok`) et par l'AC4 (`−30` proposé par le `BETWEEN`, refusé par `check`) ? L'assertion
   `expect(label).not.toContain('proposée')` rougit-elle avec le texte actuel
   (`crates/kesh-i18n/locales/fr-CH/messages.ftl:1233`, repli `frontend/src/lib/features/reconciliation/failed-proposal-label.ts:224`)
   et passe-t-elle avec le texte prescrit à l'AC7 ? Le `label` du test `failed-proposal-label.test.ts:172-178`
   peut-il contenir « proposée » par une autre voie ?
7. **Inventaire (c), T4, inventaire (a) (F7, R7, R8)** : 80 `#[sqlx::test]` (`grep -c '^#\[sqlx::test'
   crates/kesh-api/tests/reconciliation_e2e.rs`) ; chacun des lecteurs listés (`:614`, `:734`, `:849`, `:899`,
   `:1919`, `:2067`, `:3305`, `:4444`, `:5095`, helper `:3616`) est-il un `fn` qui lit `/reconciliation/proposals`,
   et la liste est-elle complète (`grep -n "reconciliation/proposals"` du même fichier) ?
   `rejeu_interblocage_e2e.rs` n'appelle-t-il vraiment que `POST /accept` ? La sortie copiée du grep
   `i.date BETWEEN` est-elle maintenant exacte ?
8. **CHANGELOG (T7) et C-28-7-13** : le texte prescrit ne parle-t-il plus du plafond de 50, et rien d'autre dans la
   fiche ne promet au CHANGELOG ou au manuel une description de ce plafond ?
9. **Recomptes** écrits par le commit : « 2 MEDIUM distincts, 14 LOW dont 10 distincts » (contre les sections
   *Bilan* des deux rapports P2), 15 mutations (`grep -cE "^\| M[0-9]+ "`), 13 titres `### C-28-7-` dont un retiré,
   11 AC, 9 tâches, ligne de `sprint-status.yaml` — **recompte depuis les fichiers, ne relis pas les nombres.**

⚠️ **Erreur de catégorie** (consigne 5 des agents) : un livrable **prescrit par la fiche mais absent du code** n'est
**pas** un défaut — la fiche est une spec, le code n'est pas encore écrit (« `InvoiceDateWindow` n'existe pas »,
« M15 n'a pas été jouée », « le manuel dit encore `peut être proposée` », « le test n'a pas l'assertion
`not.toContain` » sont **attendus**). Un finding ne porte que sur ce que **le texte du commit affirme du code
existant**, ou sur ce qu'il **prescrit de faux ou d'infaisable**. Trois fois déjà, des « MEDIUM » de Haiku ont été
de cette catégorie.
⚠️ Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée (`-F` obligatoire sur les motifs à
métacaractères). Un CRITICAL ou HIGH qui affirme qu'un patch « n'a pas été appliqué » se vérifie au `grep -nF` sur
le fichier **final** (pas sur une ligne du diff) avant d'être écrit.
⚠️ Données réelles : rien de `~/kesh-test/` (relevé réel de Guy), ni dans le rapport ni dans une commande.

## Ce que tu rends

Findings numérotés `P3-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), origine (remédiation P2 ou conception d'origine), correction proposée. ⛔ **La liste des axes
exercés ET non exercés** (les neuf ci-dessus, un par un, avec ce qui a été fait ou pourquoi pas) — un « 0 finding »
sans elle ne compte pas comme passe, et un axe déclaré exercé sans commande copiée compte comme non exercé.
Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-7-validate-p3-ciblee.md` ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`),
`make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`,
`cargo`, `npm`, `npx`, `node`, `docker`, `mysql`/`mariadb` et toute requête SQL, `gh` en écriture, `sed -i`,
`perl`, et **tout interpréteur** (`python`, `python3`, `perl`, `ruby`, `node`, `deno`, `bash`/`sh` sur un script
ou avec `-c`). Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `wc`, `git log`/`show`/`diff`/`blame`,
`gh api` en lecture, `pdftotext` vers la sortie standard (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).
