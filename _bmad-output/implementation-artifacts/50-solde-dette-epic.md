# Story 50-solde : Solde de la dette de l'Epic 50 — une seule règle de levée d'un refus de doublon (#1492)

Status: review — développée le 2026-11-24 (Sonnet 5.5, `bmad-dev-story`), gate complet vert au commit `88905d0ff` (Dev Agent Record) ; fiche écrite le 2026-11-24, réduite le même jour au lot B (#1492) par deux décisions du Project Lead ; validations P1 (Sonnet, 0/0/1/11), P2 (Opus, 0/0/1/6) et P3 (Sonnet, 0/0/2/3) ; **validation terminée** : la P3 ne rend aucun finding au-dessus de LOW sur le lot B, et sa seule remédiation (V3-3) est textuelle et locale (Change Log).

*Fiche rédigée le 2026-11-24 (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-50-solde`, branche `story/50-solde-dette-epic`, partie de
`origin/epic/50-corrections` à **`48a66ca24`**. **Tout le code cité est lu à `48a66ca24`.** Release visée : **0.20.0**, vendredi 2026-11-27
(coupure le jeudi 2026-11-26 au soir). **Aucune migration.** Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e50solde/` (inventaire
`inv-1492.txt`, régénérable par les commandes qu'il porte). Clé courte des bases : `e50s` (`kesh_e50s`, `kesh_ci_e50s`, `kesh_e2e_e50s`).*

## Story

En tant que comptable qui enregistre ses factures fournisseur,
je veux que la règle « ce refus de doublon se lève-t-il par une confirmation ? » n'ait qu'une source,
afin que le serveur ne puisse pas dire « confirmable » d'un refus qu'il maintient, ni l'inverse.

## Périmètre : la dette des revues de l'Epic 50

Story de solde (`CLAUDE.md` § *Priorités des défauts* : « seulement la dette des stories de l'epic, plafonnée par un nombre d'issues
fixé au kickoff ») ; le fichier d'épique dit « plafond fixé à la reprise » : **trois issues**, fixé par l'orchestrateur le 2026-11-24,
choisies **par priorité, puis par ancienneté**, parmi la dette ouverte par les revues des stories 48-11, 48-12a et 48-12b : #1404
(lot A), #1492 (lot B), #1414 (lot C). Deux décisions du Project Lead l'ont réduite au **seul lot B** :

- **2026-11-24, lot A (#1404) sorti vers la v0.21.0** : il exigeait une **route d'API neuve**
  (`GET /api/v1/imported-supplier-invoices/{id}`, avec sa documentation dans le guide de l'API et le manuel d'administration), touchait
  à lui seul **quatre modules** — ce qui interdit les passes ciblées — et portait l'essentiel de la charge, donc le risque pour la
  coupure de jeudi. Étude complète dans le commit `4978f345c` de cette fiche et dans `kesh-gate-logs/e50solde/inv-1404.txt`.
- **2026-11-24, lot C (#1414) sorti vers la v0.21.0** : la **preuve** du lot C (que le texte du catalogue `fr-CH` est tenu par un test)
  a **recyclé trois passes de suite** — V1-2 → V3-1 (la garde `i18n-repli-divergent-actif` ne garde pas cette clé), V2-6 → V3-2 (le
  scénario 2 de `db-resilience` ne lit pas le catalogue) — ; le défaut est un **P4 cosmétique** (un libellé) ; et la coupure du jeudi
  2026-11-26 au soir ne laisse pas une quatrième boucle. Étude complète (D-C1 avec les textes des quatre langues, inventaire, AC,
  mutations) dans le commit **`df32a86dc`** de cette fiche, dans `kesh-gate-logs/e50solde/inv-1414.txt` et dans les rapports
  `validation-p1.md` à `validation-p3.md`. **Piste simplifiée pour la v0.21.0** : le texte de D-C1 dans les quatre catalogues et le
  repli de `DegradedBanner.svelte` ; la clé `db-unavailable-banner` ajoutée à `REPLIS_A_SITE_UNIQUE` (G13,
  `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts:224`, test `les_replis_frontend_a_site_unique_suivent_le_catalogue`,
  `:351`), qui compare le repli au `fr-CH` et rend la preuve que la garde actuelle ne donne pas ; **sans** preuve E2E du catalogue
  (fragile : le rechargement d'une page coupée ne relit pas le catalogue) ; reprendre les constats **V3-1, V3-2, V3-4** (le catalogue
  se lit à l'exécution : un redémarrage du serveur, non une reconstruction) **et V3-5** (AC-6 décrivait deux mécanismes de repli
  différents) du rapport `validation-p3.md`. #1414 décrit d'ailleurs un comportement antérieur à la 48-12b.

| Issue | Labels | Origine | Sort |
|---|---|---|---|
| **#1492** | `technical-debt`, P3 | revue de code P3 de la 48-12a (P3-5) | **retenue** (lot B) |
| #1404 | `technical-debt`, P3 | validation P2 de la 48-12b (V2-9) | **sortie vers la v0.21.0** (décision du Project Lead du 2026-11-24) |
| #1414 | `bug`, `triage`, P4 | revue P1 de la 48-12b (« Comportement antérieur à la story ») | **sortie vers la v0.21.0** (décision du Project Lead du 2026-11-24) |
| #1441, #1442, #1443 | `bug`, P2 / P4 / P3 | arrondis au pair relevés à la **rédaction** de la fiche 48-11 | **écartées** : sorties du périmètre de la 48-11 par **décision de Guy du 2026-11-20** (fiche `48-11-arrondi-au-centime.md`, Change Log, ligne 793 : « **Q2** et issues : pain.001 → **#1441**, I1 → **#1442**, I2 → **#1443** » ; Dev Notes, ligne 590) ; ce ne sont pas des constats de revue |
| #1415, #1416 | `bug`, P4 | revue de la 48-12a (formes Unicode du numéro) | **écartées** : P4, sous le plafond |
| #1453 | `documentation`, P4 | revue de la 48-12b (doc-comment de `unknown-outcome.ts`) | **écartée** : P4, sous le plafond |
| #1491 | `bug`, P4 | revue de la 48-12a (libellé d'audit) | **écartée** : P4, sous le plafond |

## Ce qui se lit au code (`48a66ca24`)

### Lot B — #1492 : la levée d'un refus de doublon jugée deux fois

1. **`kesh-db`** — `crates/kesh-db/src/repositories/supplier_invoices.rs:658-664`, dans `guard_duplicates_in_tx` (`:622`) :
   ```rust
   let liftable = match (finding.rule, gesture) {
       (DuplicateRule::SameNumberSameYear, DuplicateGesture::Entry) => false,
       (DuplicateRule::SameDateSameTotal, _) | (_, DuplicateGesture::Unvalidation) => true,
   };
   if liftable && policy == DuplicatePolicy::Confirmed {
   ```
   puis le refus, `:665-673` (`SameNumberSameYear` → `DbError::SupplierInvoiceDuplicate { existing, gesture }`, `SameDateSameTotal` →
   `DbError::SupplierInvoiceProbableDuplicate { existing, gesture }`). Trois appelants : `:944`, `:1152`, `:1875` (ce dernier avec
   `DuplicateGesture::Unvalidation`, `:1883`). `DuplicateRule` et sa méthode `number` : `:484-502` (`impl DuplicateRule` à `:494`).
2. **`kesh-api`** — `crates/kesh-api/src/errors.rs:6253-6260`, `supplier_invoice_duplicate_response(code, existing, gesture, probable)` :
   `let confirmable = probable || gesture == DuplicateGesture::Unvalidation;` (`:6260`), écrit dans `details.confirmable` (`:6283`) ;
   le message se choisit sur `(gesture, probable)` (`:6261-6274`) ; appelé par les deux bras `:5583-5598` avec `probable` à `false` puis
   `true`.
3. Les deux tables concordent aujourd'hui (quatre cases : (règle 1, saisie) → non ; (règle 2, saisie) → oui ; (règle 1 ou 2,
   dévalidation) → oui) ; rien ne les lie. **L'écran ne décide rien** : il lit `details.confirmable`
   (`frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts:592-603`, `:619`) — il n'est pas un troisième site.
4. **Les tests qui tiennent les cases aujourd'hui** : `crates/kesh-db/tests/supplier_invoice_duplicate.rs` (refus levé ou non ; pour
   (règle 2, dévalidation) : `:1136` vérifie le refus sous `REFUSE`, `:1139-1141` la levée sous `CONFIRMED`,
   `repasser(&pool, &seeded, a, qui, CONFIRMED).await.expect("confirmé")`) ; `crates/kesh-api/tests/supplier_invoice_duplicate_e2e.rs`,
   test `create_twice_is_refused_with_details_and_message` : boucle `for confirm in [false, true]` (`:226`), `:231`
   `assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DUPLICATE"), "{e}");`, puis l'`assert_eq!` multiligne `:232-240` sur
   `e["error"]["details"]` (le littéral `"confirmable": false` est à `:238`, mais un échec se signale à l'invocation de la macro,
   **`:232`** — validation P2, V2-1), règle 1 à la saisie ;
   `:263` (règle 2 à la saisie, `true`), `:421` (règle 1 à la dévalidation, `true`). **Fait établi** (validation P1, V1-5,
   `grep -n confirmable` → `:4, :212, :213, :238, :263, :375, :421` ; `grep -nF PROBABLE` → `:260` seul) : **la case (règle 2, dévalidation)
   n'est assertée par aucun test HTTP** ; l'AC-2 est seule à la tenir côté `details.confirmable`. Un module de tests unitaires existe dans `kesh-api/src/errors.rs`
   (`:6587`), avec l'aide `response_body(resp: Response) -> (StatusCode, serde_json::Value)` (`:6605`) ; un autre dans
   `supplier_invoices.rs` (`:3838`).

**Inventaire par le symptôme** (« une décision sur la levée d'un refus de doublon », `inv-1492.txt`) :
`git grep -nE "liftable|confirmable|DuplicateGesture::Unvalidation|probable \|\|" -- 'crates/*/src/*'`,
`git grep -nE "confirmable" -- frontend/src ':!*.test.ts'` et, pour les récitations écrites sans ces mots (validation P1, V1-4),
`git grep -niE "jamais.{0,40}(règle 1|SUPPLIER_INVOICE_DUPLICATE)|lève.{0,30}(deux règles|règle 2)|les deux règles|levé \(règle|aucune\** sortie|sans sortie" -- 'crates/*/src/*'`
(insensible à la casse et élargie à « aucune sortie » après la validation P2, V2-2 ; sorties ajoutées à `inv-1492.txt`). **Sites qui décident** : deux, `supplier_invoices.rs:658-661` et
`errors.rs:6260` — tous deux corrigés (D-B1, D-B2). **Sites qui lisent ou décrivent** : `supplier_invoices.rs:1883` (passe le geste,
ne décide pas) ; doc-comments et commentaires qui récitent la table — `kesh-db/src/errors.rs:574`, `:577-578` (« la règle 1 … n'a
**aucune** sortie »), `:580-581`, `:1892-1894`, `:1905`, `supplier_invoices.rs:449-462`, `:1566-1569` (« qui lève **les deux règles** »),
`:1871-1873` (« Les deux règles se lèvent par confirmation »), `kesh-api/src/errors.rs:5578-5581`, `:6240`, `:6247-6251`, `kesh-api/src/routes/supplier_invoices.rs:349-353`
(« il ne lève **jamais** `SUPPLIER_INVOICE_DUPLICATE` »), `:669-673` (« les deux règles »), `:749-750` (« levé (règle 2 seulement) »),
`:956-957` — réécrits pour **nommer la source** (D-B3) ;
frontend (`supplier-invoice-form.ts`, `SupplierInvoiceForm.svelte:1193`, `[id]/+page.svelte:1016`) : lit le booléen, inchangé. Les
autres « confirmable » du relevé (`kesh-import`, `payment_batches.rs:351`, `errors.rs:1073`, `:1096`, `BankImportUpload.svelte:488`)
parlent d'autres refus : hors classe. **Hors code, contrat** : `docs/api-external.md:1028`, `:1315-1316` et
`docs/manual/fr/admin-manual.tex:2585` récitent la table comme contrat de l'API ; ils sont justes et le restent (la story ne change
aucune case) — non touchés.

### Manuels

`docs/manual/fr/admin-manual.tex:2585` (section `sec:maj-0-20-0`) décrit `confirmable` (« jamais le premier à la création, au
\texttt{PUT} et à la complétion …, toujours le second, et les deux au repassage d'une annulée ») : **reste vrai**, la story ne change
aucune case. `docs/api-external.md:1028`, `:1315-1316` : idem. Aucun autre manuel (`ls docs/manual/*/` : seul `fr` porte des `.tex` ;
PDF aplatis par `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, grep de `confirmable`, `confirmNotDuplicate` : seulement ce paragraphe).
**Aucun manuel ne change.**

## Décisions

### Lot B — #1492

- **D-B1 — la table vit dans `kesh-db`, en un seul endroit** : une méthode publique
  `DuplicateRule::lifted_by_confirmation(self, gesture: DuplicateGesture) -> bool` dans
  `crates/kesh-db/src/repositories/supplier_invoices.rs` (dans `impl DuplicateRule`, `:494`), qui porte le `match` de `:658-661`
  **tel quel** et le doc-comment de la table (quatre cases, D1/D6bis de la 48-12a). `guard_duplicates_in_tx` l'appelle
  (`if finding.rule.lifted_by_confirmation(gesture) && policy == DuplicatePolicy::Confirmed`). Le `match` reste **exhaustif sur le
  couple** (pas de `_ => …` qui avalerait un geste ou une règle ajoutés demain).
- **D-B2 — `kesh-api` relit la table, il ne la réécrit pas** : `supplier_invoice_duplicate_response(existing, gesture, rule:
  DuplicateRule)` remplace `(code, existing, gesture, probable)` ; **le code d'erreur se déduit de la règle dans la fonction**, par un
  seul `match rule` exhaustif (`SameNumberSameYear` → `"SUPPLIER_INVOICE_DUPLICATE"`, `SameDateSameTotal` →
  `"SUPPLIER_INVOICE_PROBABLE_DUPLICATE"`) — l'appariement code ↔ règle n'est plus écrit deux fois par bras (validation P1, V1-6) ;
  `confirmable = rule.lifted_by_confirmation(gesture)` ; le message se choisit sur `(gesture, rule)`. Les deux bras
  (`errors.rs:5583-5598`) ne passent plus que `DuplicateRule::SameNumberSameYear` et `DuplicateRule::SameDateSameTotal`. **Ce que le
  compilateur ne tient pas** : le choix de la règle dans chaque bras (variante → règle) ; il est tenu par l'AC-2 (code **et**
  `confirmable` sur les quatre cases, à partir des variantes de `DbError`) et par la mutation M-B3, non par le typage.
  **Options écartées** : un champ `confirmable` dans les deux variantes de `DbError` (redondant avec `gesture` : un état
  incohérent devient constructible, et les tests de `kesh-db` qui comparent des `Debug` de variantes changent tous) ; une variante unique
  `SupplierInvoiceDuplicate { rule, existing, gesture }` (change `error_code`, `kesh-db/src/errors.rs:2606-2607`, et une vingtaine de
  motifs de tests : hors de la mesure d'une story de solde).
- **D-B3 — les doc-comments nomment la source** : ceux qui récitent la table (`kesh-db/src/errors.rs:574`, `:577-578`, `:580-581`,
  `:1892-1894`, `:1905` ; `supplier_invoices.rs:449-462`, `:1566-1569`, `:1871-1873` ; `kesh-api/src/errors.rs:5578-5581`, `:6240`, `:6247-6251` ;
  `kesh-api/src/routes/supplier_invoices.rs:349-353`, `:669-673`, `:749-750`, `:956-957`) gardent leur résumé et renvoient à
  `DuplicateRule::lifted_by_confirmation` — sans en changer le sens. `routes/supplier_invoices.rs` est dans `kesh-api`, module déjà
  compté.

## Critères d'acceptation

1. **AC-1 — la table, une fois** (`supplier_invoices.rs`, module `tests`) : **quatre tests purs, un par case**, de
   `DuplicateRule::lifted_by_confirmation`, nommés `lifted_regle1_saisie` (→ `false`), `lifted_regle2_saisie` (→ `true`),
   `lifted_regle1_devalidation` (→ `true`), `lifted_regle2_devalidation` (→ `true`) ; chacun une seule assertion, message
   « table (règle N, geste) ».
2. **AC-2 — la réponse relit la table** (`kesh-api/src/errors.rs`, module `tests`, avec `response_body`) : **quatre
   `#[tokio::test]`, un par case** (pas de boucle : un test s'arrête à sa première assertion en échec, et la sortie de cargo doit montrer
   chaque case — validation P2, V2-1), nommés `duplicate_response_regle1_saisie`, `…_regle2_saisie`, `…_regle1_devalidation`,
   `…_regle2_devalidation`. Chacun construit `AppError::Database(DbError::Supplier…Duplicate { existing, gesture })` et enchaîne,
   **dans cet ordre imposé**, des `assert_eq!` distincts, chacun avec un message qui nomme la case et sa nature :
   (1) statut `409` et `error.code` (message « code (règle N, geste) ») ; (2) l'assertion **relue**,
   `details.confirmable == DuplicateRule::<règle>.lifted_by_confirmation(gesture)` (message « relue (règle N, geste) ») ;
   (3) l'assertion **littérale**, `details.confirmable == <true|false>` (message « littérale (règle N, geste) ») ; (4) le message
   d'aujourd'hui pour la case (clés `error-supplier-invoice-duplicate`, `-probable-duplicate`, `-duplicate-unvalidate` ; message
   « texte (règle N, geste) »). L'ordre fait qu'une assertion « verte » se **lit** dans la sortie : elle est verte si une assertion
   **postérieure** du même test a paniqué (§ *Tests*).
3. **AC-3 — plus aucune seconde table** :
   `git grep -nE "let liftable|probable \|\||gesture == DuplicateGesture::Unvalidation" -- 'crates/*/src/*'` est **vide** ;
   `git grep -nw lifted_by_confirmation -- 'crates/*/src/*'` ne rend que la définition, son appel dans `guard_duplicates_in_tx`, son
   appel dans `supplier_invoice_duplicate_response`, les doc-comments de D-B3 et les tests. Sortie copiée au Dev Agent Record.
4. **AC-4 — rien ne change au comportement** : `crates/kesh-db/tests/supplier_invoice_duplicate.rs`,
   `supplier_invoice_duplicate_lock.rs` et `crates/kesh-api/tests/supplier_invoice_duplicate_e2e.rs` passent **sans modification**
   (`git diff origin/epic/50-corrections...HEAD --stat -- <ces trois fichiers>` vide). Aucun E2E neuf : rien ne change à la frontière
   HTTP ni dans le navigateur ; les e2e existants de l'API (`:232`, `:263`, `:421`) jouent déjà la réponse.
5. **AC-5 — aucune facture, aucun avoir, aucune écriture ne change** : aucun fichier de `crates/kesh-qrbill`, `crates/kesh-report`,
   `crates/kesh-core`, ni `crates/kesh-db/migrations/` n'est touché (`git diff origin/epic/50-corrections...HEAD --stat --
   crates/kesh-qrbill crates/kesh-report crates/kesh-core crates/kesh-db/migrations` vide) ; les seuls changements de `kesh-db` sont ceux
   de D-B1/D-B3, sans SQL modifié (`git diff origin/epic/50-corrections...HEAD -- crates/kesh-db | grep -E '^[+-].*(SELECT|UPDATE|INSERT|DELETE)'`
   vide) ; l'audit `supplier_invoice.duplicate_confirmed` s'écrit dans les mêmes cas qu'avant (AC-4) ; aucun fichier de `frontend/` ni
   de `crates/kesh-i18n` n'est touché.
6. **AC-6 — gates** (§ *Gates*) verts au dernier commit de code ; E2E complet « vert » au sens du `CLAUDE.md` (seuls les échecs
   attendus, jugés fichier par fichier).

*(Numérotation du 2026-11-24 après la sortie du lot C : AC-1 à AC-4 inchangés ; les anciens AC-5 à AC-8 du lot C sont retirés ;
l'ancien AC-9 devient AC-5, l'ancien AC-10 devient AC-6.)*

## Tests à écrire, et la preuve du rouge

Journaux attendus sous `/home/gcorbaz/devel/kesh-gate-logs/e50solde/dev/`.

| Test | Rouge sans le correctif | Preuve |
|---|---|---|
| AC-1 | — (la méthode est neuve : les tests ne compilent pas avant elle) | M-B1 et M-B2, ci-dessous |
| AC-2 | ne rougit pas sur le code d'avant (les valeurs sont les mêmes) : **filet**, et c'est sa raison d'être — il rougit sous M-B1, M-B2, M-B3 | `mut-b*.log` |
| AC-3, AC-4, AC-5 | — | commandes et sorties copiées |

**Ce que la sortie montre, et comment on y lit une assertion « verte »** (validation P2, V2-1). Un `assert_eq!` qui passe n'écrit
rien ; cargo n'affiche que **l'assertion qui panique** : `thread '<module>::<test>' panicked at <fichier>:<ligne>:<colonne>:`, puis
`assertion `left == right` failed: <message>` et les valeurs `left` / `right` (pour un `.expect`, `<message>: <erreur>`). nextest
nomme chaque test par son binaire et son **chemin de module complet** : `kesh-db repositories::supplier_invoices::tests::lifted_regle1_saisie`,
`kesh-api errors::tests::duplicate_response_regle1_saisie`, `kesh-api::supplier_invoice_duplicate_e2e create_twice_is_refused_with_details_and_message`,
`kesh-db::supplier_invoice_duplicate rule_1_same_number_same_year_is_refused_without_exit` (les noms du tableau ci-dessous en sont le
**suffixe** ; le préfixe exact est celui que nextest imprime, à recopier au journal). Chaque journal `mut-*.log` **nomme donc, par test,
l'assertion qui échoue** — fichier:ligne et message —, et rien d'autre ; une assertion se dit « passée » seulement si une assertion
**postérieure du même test** a paniqué (ordre imposé de l'AC-2 : code, relue, littérale, texte), ou si le test entier est listé `PASS`.

**Les mutations du lot B.** Une mutation qui ne touche que le corps de `lifted_by_confirmation` distingue « une table » de
« deux tables » par **l'assertion qui panique**. Le tableau donne, par test, l'échec attendu tel que cargo l'imprime ; la dernière
colonne dit ce qu'on verrait si `kesh-api` gardait sa propre table — c'est ce qu'il faut **ne pas** voir.

| Mutation | Test (suffixe du nom nextest) | Échec attendu (une table) | Ce que montrerait l'hypothèse « deux tables » |
|---|---|---|---|
| **M-B1** : (règle 1, `Entry`) → `true`, seul `supplier_invoices.rs` touché (`mut-b1.log`) | `kesh-db` `…::tests::lifted_regle1_saisie` | panique, message « table (règle 1, Entry) » | idem |
| | `kesh-api` `errors::tests::duplicate_response_regle1_saisie` | panique à l'assertion **littérale**, message « littérale (règle 1, Entry) », `left: true`, `right: false` — les assertions « code » et « relue », antérieures, sont donc passées | panique à l'assertion **relue**, message « relue (règle 1, Entry) », `left: false`, `right: true` |
| | les trois autres tests `duplicate_response_*` | `PASS` | `PASS` |
| | e2e `create_twice_is_refused_with_details_and_message` | panique à `tests/supplier_invoice_duplicate_e2e.rs:232`, `left` portant `"confirmable": Bool(true)` — à la **première** itération (`confirm = false`) : `:231`, antérieure, est passée ; l'itération `confirm = true` n'est pas atteinte | panique à `:231` à la **seconde** itération : `left: (201, …)`, `right: (409, "SUPPLIER_INVOICE_DUPLICATE")` |
| | `kesh-db` `tests/supplier_invoice_duplicate.rs` : `rule_1_same_number_same_year_is_refused_without_exit` (`:206`, cas `(" FA2026-1\u{200B}", CONFIRMED)`, `:222`) et `update_replays_the_rule_only_when_one_of_the_four_changes` (`:674`, boucle `for policy in [REFUSE, CONFIRMED]`, `:762`) | **paniquent aussi** sur l'itération `CONFIRMED` : la règle 1 levée à la saisie, l'opération réussit là où le test attend le refus (ligne et message à relever au journal) — **même mutation, hors discrimination** : attendus, ce n'est pas une mutation qui « fuit » (validation P3, V3-3) | idem (la mutation est dans `kesh-db` dans les deux hypothèses) |
| **M-B2** : (règle 2, `Unvalidation`) → `false`, seul `supplier_invoices.rs` touché (`mut-b2.log`) | `kesh-db` `…::tests::lifted_regle2_devalidation` | panique, message « table (règle 2, Unvalidation) » | idem |
| | `kesh-api` `errors::tests::duplicate_response_regle2_devalidation` | panique à l'assertion **littérale**, « littérale (règle 2, Unvalidation) », `left: false`, `right: true` | panique à l'assertion **relue**, « relue (règle 2, Unvalidation) » |
| | `kesh-db` `tests/supplier_invoice_duplicate.rs`, `unvalidation_rule_2_other_cancelled_and_validated_invoices` (`:1126`) | panique au `.expect("confirmé")` (`:1139-1141` ; la ligne exacte, `:1139` ou `:1141`, est celle que rustc imprime : la relever au journal), message `confirmé: SupplierInvoiceProbableDuplicate …` — l'assertion `:1136`, antérieure, est passée | idem (non discriminant, et la fiche ne le prétend pas) |
| | aucun test HTTP | — (fait établi, point 4) | — |
| **M-B3** : dans `kesh-api/src/errors.rs`, les règles des deux bras `DbError::SupplierInvoiceDuplicate` / `…ProbableDuplicate` échangées (`mut-b3.log`) | les **quatre** tests `duplicate_response_*` | chacun panique à son assertion **code**, message « code (règle N, geste) », le code reçu étant l'autre (le code se déduit de la règle, D-B2) | — |

Après chaque mutation restaurée, **toucher le fichier** avant de relancer (cargo peut garder le binaire muté).

## Tâches

1. **Étape 0** : relire à la tête de la branche les lignes citées (`supplier_invoices.rs:449-462`, `:484-502`, `:622-674`, `:1566-1569`,
   `:1871-1886` ; `kesh-db/src/errors.rs:570-585`, `:1885-1912`, `:2606-2607` ; `kesh-api/src/errors.rs:5576-5598`, `:6230-6290`,
   `:6587-6620` ; `routes/supplier_invoices.rs:349-353`, `:669-673`, `:749-750`, `:956-957` ; `kesh-db/tests/supplier_invoice_duplicate.rs:206-230`,
   `:674`, `:762`, `:1126-1141` ; `kesh-api/tests/supplier_invoice_duplicate_e2e.rs:217-240`) ; en copier trois au Change Log ;
   régénérer l'inventaire (commandes de `inv-1492.txt`) et noter tout écart (une reprise de `main` dans `epic/50-corrections` est en
   cours : un site neuf entre au relevé avant tout code).
2. `kesh-db` : `lifted_by_confirmation`, appel dans `guard_duplicates_in_tx`, doc-comments (D-B3) ; tests AC-1.
3. `kesh-api` : `supplier_invoice_duplicate_response` (D-B2), bras, doc-comments et commentaires (D-B3) ; tests AC-2.
4. Mutations M-B1 à M-B3, journaux ; **gate complet** au commit qui porte le patch de `kesh-db` (exception `kesh-db` du `CLAUDE.md` :
   il touche un repository).
5. Propagation (AC-3), AC-4, AC-5 ; Dev Agent Record. **Ne pas écrire `CHANGELOG.md` ni `sprint-status.yaml`** (l'orchestrateur tient
   les statuts).

Un commit par étape franchie (test rouge, correctif vert, remédiation), poussé sur `story/50-solde-dette-epic` seule (`CLAUDE.md`
§ *Worktrees et agents*), chacun avec `refs #1492` ; **aucun mot-clé de fermeture**.

## Gates

- **En cours de développement** (gate ciblé) : `cargo fmt --all -- --check` ;
  `cargo clippy --workspace --all-targets -- -D warnings` ;
  `cargo nextest run -E 'binary(supplier_invoice_duplicate_e2e) | binary(supplier_invoice_duplicate) | binary(supplier_invoice_duplicate_lock) | (package(kesh-api) & kind(lib)) | (package(kesh-db) & kind(lib))'`
  sur `kesh_e50s`. **Exception `kesh-db`** : la story touche un repository → **gate complet** au commit qui le porte, même en cours de
  boucle. Aucun fichier du frontend n'est touché : les contrôles du frontend ne tournent qu'au gate complet.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh` (base `kesh_ci_e50s`).
- **À la clôture**, au dernier commit de code : `gate-ci` et **E2E complet** (`docs/testing.md` § *Prérequis Playwright local*,
  `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés, base `kesh_e2e_e50s` ; « vert » = les seuls échecs attendus, jugés
  fichier par fichier).
- Une commande lourde à la fois sous `scripts/mem-guard.sh`, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant, verrou
  `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`. Le story file n'affirme que ce qui a tourné.

## Note de version

**Interne**

- La règle qui dit si une confirmation lève un refus de doublon de facture fournisseur n'est plus écrite qu'une fois (refs #1492).

## Modules — règle de splitting

Grain de la 48-12b (une crate backend, un domaine ou une route du frontend, ou la documentation ; les tests suivent leur module) :

| # | Module | Fichiers de production |
|---|---|---|
| 1 | `kesh-db` | `repositories/supplier_invoices.rs`, `errors.rs` (doc-comments) |
| 2 | `kesh-api` | `errors.rs`, `routes/supplier_invoices.rs` (commentaires, D-B3) |

**Deux modules : sous la borne de cinq, pas de découpage.** Aucune documentation, aucun manuel, aucun fichier du frontend ne change.
Une remédiation qui touche les deux modules interdit la passe ciblée (`CLAUDE.md` § *La passe ciblée*) ; une remédiation de `kesh-db`
impose le gate complet.

## Dev Notes

- **Charge révisée** : **deux à trois heures** de développement — la méthode et son appel, la fonction de réponse, une douzaine de
  doc-comments, huit tests neufs, trois mutations (chacune recompile `kesh-db` puis `kesh-api`) et le gate complet qu'impose
  l'exception `kesh-db` ; s'y ajoutent `gate-ci`, l'E2E complet et la boucle de revue (au moins deux passes alternées). **Calendrier** :
  le développement démarre le mercredi 2026-11-25 au matin ; la clôture avant la coupure du jeudi 2026-11-26 au soir est tenable. Si
  elle ne l'est pas, la story sort entière (décision du Project Lead).
- **Fichiers communs** : avec la 48-12a (close, fusionnée dans `epic/50-corrections`) — `supplier_invoices.rs`, `kesh-api/src/errors.rs`,
  `routes/supplier_invoices.rs`. Un autre agent reprend `main` dans `epic/50-corrections` : **l'étape 0 relit les lignes citées après
  cette reprise**.
- **Ce qui doit rester vrai** : `details.confirmable`, le code d'erreur et le message aux mêmes valeurs dans les quatre cases ; l'audit
  `supplier_invoice.duplicate_confirmed` dans les mêmes cas ; aucun test existant modifié.
- **Hors périmètre** : #1404 (lot A) et #1414 (lot C), sortis vers la v0.21.0 (§ *Périmètre*, avec la piste du lot C) ; la pagination
  en mémoire de l'outil MCP `kesh_pieces`, portée par **#1509** ; #1415, #1416, #1453, #1491 (P4, sous le plafond) ; #1441, #1442,
  #1443 (décision de Guy du 2026-11-20, Q2 de la 48-11).
- **Références** : fiche `48-12a-doublons-fournisseurs-serveur.md` (D1, D5, D6bis, D7) ; `docs/testing.md` ;
  `docs/agents-orchestration.md`.

## Dev Agent Record

### Agent Model Used

Sonnet 5.5 (`claude-sonnet-5-5`), en autonomie, le 2026-11-24 ; worktree `/home/gcorbaz/devel/kesh-50-solde`, branche
`story/50-solde-dette-epic`, partie de `epic/50-corrections` à `48a66ca24` (non reprise : l'orchestrateur fusionnera). Bases
`kesh_e50solde`, `kesh_e2e_e50solde` (créée, non utilisée : pas d'E2E), `kesh_ci_e50solde`. Aucun droit MariaDB accordé.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e50solde/dev/` : `inv-etape0.txt` (inventaire par le symptôme), `rouge-ac1-ac2.log`
(rouge), `clippy-1.log`, `vert-cible.log` (gate ciblé), `mut-b1.log` / `mut-b2.log` / `mut-b3.log` (assertions qui échouent) avec
`mut-bN.full.log` (sortie nextest entière, diff restauré) et `mut-bN.diff.txt` (la mutation), `test-fast-ci.log`, `gate-ci.log` et
`gate-ci-journal.log`. Alerte de file d'attente : `/home/gcorbaz/devel/kesh-gate-logs/e50solde/alerte.md`.

### Completion Notes List

**Étape 0** (à `02cb59a1d`, la base de la branche est restée `48a66ca24` : `git fetch` n'a rien fait bouger d'`epic/50-corrections`).
Lignes relues au code, identiques à la fiche :
- `supplier_invoices.rs:658-664` : `let liftable = match (finding.rule, gesture) { (DuplicateRule::SameNumberSameYear, DuplicateGesture::Entry) => false, (DuplicateRule::SameDateSameTotal, _) | (_, DuplicateGesture::Unvalidation) => true, };` puis `if liftable && policy == DuplicatePolicy::Confirmed {`.
- `kesh-api/src/errors.rs:6260` : `let confirmable = probable || gesture == DuplicateGesture::Unvalidation;`.
- `kesh-api/src/errors.rs:5583-5598` : les deux bras passent `false` puis `true`.
Inventaire régénéré par les commandes de `inv-1492.txt` : même ensemble de sites, aucun site neuf (`inv-etape0.txt`).

**Livré** (D-B1, D-B2, D-B3) :
- `DuplicateRule::lifted_by_confirmation(self, gesture) -> bool` (`kesh-db`, `supplier_invoices.rs`), le `match` de `:658-661` repris tel
  quel, exhaustif sur le couple, doc-comment de la table à quatre cases ; `guard_duplicates_in_tx` l'appelle (l'import local de
  `DuplicateGesture`, devenu inutile, est retiré).
- `supplier_invoice_duplicate_response(existing, gesture, rule)` (`kesh-api/src/errors.rs`) : le code se déduit de la règle par un seul
  `match rule`, `confirmable = rule.lifted_by_confirmation(gesture)`, le message se choisit sur `(gesture, rule)` ; les deux bras ne
  passent plus que la règle.
- Doc-comments et commentaires de D-B3 renvoyant à la source : `kesh-db/src/errors.rs` (`DuplicateGesture`, ses deux variantes,
  `SupplierInvoiceDuplicate`, `SupplierInvoiceProbableDuplicate`), `supplier_invoices.rs` (`DuplicatePolicy`, `unvalidate`, le commentaire
  « 6 bis »), `kesh-api/src/errors.rs` (commentaire des bras, doc-comment de la fonction), `routes/supplier_invoices.rs` (quatre sites).
  Les doc-comments qui décrivent les **textes** (`…_UNVALIDATE_FR`, « pour les deux règles ») ne décident rien et sont restés.
- Tests neufs : quatre purs `lifted_regle{1,2}_{saisie,devalidation}` (AC-1) ; quatre `#[tokio::test]`
  `duplicate_response_regle{1,2}_{saisie,devalidation}` (AC-2), assertions dans l'ordre imposé code, relue, littérale, texte.

**Rouge** (`rouge-ac1-ac2.log`, commit `cf8c2c01a`) : la méthode n'existant pas, les tests ne compilent pas (`error[E0599]: no method named
lifted_by_confirmation` × 8 : quatre dans `kesh-api/src/errors.rs`, quatre dans `kesh-db/.../supplier_invoices.rs`). C'est la preuve
annoncée au tableau de la fiche (AC-1 : « les tests ne compilent pas avant la méthode » ; AC-2 : filet qui ne rougit pas sur l'ancien
code, rougit sous M-B1 à M-B3). Le rouge s'est pris par `cargo test --no-run` (compilation seule, sans verrou), la file des gates
des autres agents ayant retenu la prise plus de 25 minutes ; le gate ciblé vert a ensuite tourné sous verrou.

**Vert ciblé** (`vert-cible.log`) : `cargo nextest run -E 'binary(supplier_invoice_duplicate_e2e) | binary(supplier_invoice_duplicate) | binary(supplier_invoice_duplicate_lock) | (package(kesh-api) & kind(lib)) | (package(kesh-db) & kind(lib))'` sur `kesh_e50solde` : **1259 tests, 1259 passés, 5 ignorés** ; `cargo fmt --all` sans reste, `cargo clippy --workspace --all-targets -- -D warnings` vert (`clippy-1.log`).

**Mutations** (`run-mut.sh` du scratchpad : `mut.py` applique, nextest rejoue `test(lifted_regle) | test(duplicate_response_regle) | binary(supplier_invoice_duplicate_e2e) | binary(supplier_invoice_duplicate)` en `--no-fail-fast`, `git checkout -- <fichier>` puis `touch`, `git diff` compté : **0 ligne après chacune**). Assertion qui échoue, comparée à l'attendu de la fiche :

| Mutation | Test | Assertion qui échoue (journal) | Attendu de la fiche |
|---|---|---|---|
| **M-B1** (règle 1, `Entry` → `true`) — 27 tests, 22 passés, 5 échoués | `kesh-db …::tests::lifted_regle1_saisie` | `supplier_invoices.rs:3923:9`, « table (règle 1, Entry) » | conforme |
| | `kesh-api errors::tests::duplicate_response_regle1_saisie` | `errors.rs:6660:9`, « littérale (règle 1, Entry) », `left: Bool(true)`, `right: false` ; « code » et « relue » antérieures passées | **conforme (une table)** : littérale, non « relue » |
| | les trois autres `duplicate_response_*` | PASS | conforme |
| | e2e `create_twice_is_refused_with_details_and_message` | `supplier_invoice_duplicate_e2e.rs:232:9`, `left` portant `"confirmable": Bool(true)`, `right` `false` ; `:231` passée | conforme (`:232`, première itération) |
| | `kesh-db::supplier_invoice_duplicate rule_1_same_number_same_year_is_refused_without_exit` et `update_replays_the_rule_only_when_one_of_the_four_changes` | `supplier_invoice_duplicate.rs:238:10` (message `" FA2026-1\u{200b}": SupplierInvoiceWithLines { … }`, l'opération a réussi) et `:777:10` (« doublon préexistant, date changée: SupplierInvoiceWithLines … ») | conforme : paniquent aussi, hors discrimination ; lignes `:238` et `:777` (l'`expect` de l'itération `CONFIRMED`, non les lignes `:222`/`:762` de la boucle) |
| **M-B2** (règle 2, `Unvalidation` → `false`) — 27 tests, 24 passés, 3 échoués | `kesh-db …::tests::lifted_regle2_devalidation` | `supplier_invoices.rs:3955:9`, « table (règle 2, Unvalidation) » | conforme |
| | `kesh-api errors::tests::duplicate_response_regle2_devalidation` | `errors.rs:6745:9`, « littérale (règle 2, Unvalidation) », `left: Bool(false)`, `right: true` | **conforme (une table)** |
| | `kesh-db::supplier_invoice_duplicate unvalidation_rule_2_other_cancelled_and_validated_invoices` | `supplier_invoice_duplicate.rs:1141:10`, `confirmé: SupplierInvoiceProbableDuplicate { existing: ExistingSupplierInvoice { id: 2, … }, gesture: Unvalidation }` (la ligne exacte est `:1141`) ; `:1136` passée | conforme |
| **M-B3** (règles des deux bras échangées) — 27 tests, 17 passés, 10 échoués | les **quatre** `duplicate_response_*` | chacun à son assertion « code (règle N, geste) » : `errors.rs:6651:9` (règle 1, Entry), `:6679:9` (règle 2, Entry), `:6707:9` (règle 1, Unvalidation), `:6736:9` (règle 2, Unvalidation) ; le code reçu est l'autre | conforme |
| | **écart à la fiche, non attendu** : six tests de `supplier_invoice_duplicate_e2e` rougissent aussi (`create_twice_is_refused_with_details_and_message` à `:231`, `completion_refuses_a_duplicate_and_keeps_the_document` `:365`, `put_and_api_key_paths_apply_the_rule` `:302`, `completion_route_waits_for_the_supplier_then_refuses` `:559`, `put_route_takes_the_supplier_lock_before_reading_the_document` `:525`, `unvalidation_of_a_cancelled_duplicate_by_api_key` `:418`) | la fiche ne listait que les quatre tests unitaires pour M-B3 ; les e2e voient le code échangé, comme il se doit | écart déclaré, sans conséquence : plus de rouges, non moins |

**Ce qui est tenu, ce qui ne l'est pas.** Les deux « relues » sont tenues par la mutation M-B1/M-B2 (elles ne rougissent pas : la
relue suit la table mutée, c'est la littérale qui tombe — la lecture « une seule table » de la fiche) ; si `kesh-api` gardait sa
propre table, c'est la « relue » qui tomberait, ce que l'ordre des assertions fait lire. Le choix variante → règle de chaque bras reste tenu
par M-B3 et par l'AC-2, non par le typage (limite déclarée à D-B2).

**AC** :
- **AC-1, AC-2** : 8 tests neufs verts (`vert-cible.log` : `kesh-db …::lifted_regle{1,2}_{devalidation,saisie}`, `kesh-api errors::tests::duplicate_response_regle…`).
- **AC-3** : `git grep -nE "let liftable|probable \|\||gesture == DuplicateGesture::Unvalidation" -- 'crates/*/src/*'` : **vide**.
  `git grep -nw lifted_by_confirmation -- 'crates/*/src/*'` : la définition (`supplier_invoices.rs:517`), son appel dans
  `guard_duplicates_in_tx` (`:679`), son appel dans `supplier_invoice_duplicate_response` (`errors.rs:6265`), les huit appels des tests
  (`errors.rs:6657, 6685, 6713, 6742` ; `supplier_invoices.rs:3925, 3935, 3945, 3955`) et les doc-comments de D-B3
  (`errors.rs:5581, 6248` ; `routes/supplier_invoices.rs:353, 672, 751, 959` ; `kesh-db/errors.rs:575, 581, 585, 1900, 1912` ; `supplier_invoices.rs:461, 1586, 1891`) — rien d'autre.
- **AC-4** : `git diff 48a66ca24...HEAD --stat -- crates/kesh-db/tests/supplier_invoice_duplicate.rs crates/kesh-db/tests/supplier_invoice_duplicate_lock.rs crates/kesh-api/tests/supplier_invoice_duplicate_e2e.rs` : **vide** ; les trois binaires passent (`vert-cible.log`, `test-fast-ci.log`, `gate-ci-journal.log`).
- **AC-5** : `git diff 48a66ca24...HEAD --stat -- crates/kesh-qrbill crates/kesh-report crates/kesh-core crates/kesh-db/migrations frontend crates/kesh-i18n` : **vide** ; `git diff 48a66ca24...HEAD -- crates/kesh-db | grep -E '^[+-].*(SELECT|UPDATE|INSERT|DELETE)'` : **vide**.
- **AC-6** : gates ci-dessous ; **l'E2E complet n'a pas tourné** (consigne : il tourne à la clôture, après la revue).
- Contrôle des mots-clés de fermeture contre `origin/epic/50-corrections` : vide.

**Gates réellement exécutés**, au dernier commit de code `88905d0ff`, bases remises à neuf (`kesh_e50solde` supprimée, recréée, 95 migrations par `sqlx migrate run`, seed) :
- `scripts/test-fast.sh --ci` (fmt, clippy `-D warnings`, nextest profil `ci`) : **6270 tests, 6270 passés, 15 ignorés, 388,5 s, vert** (`test-fast-ci.log`).
- `scripts/gate-ci.sh kesh_ci_e50solde` : **6289 passés, 0 échoués, 19 ignorés, code 0** (`gate-ci.log`, `gate-ci-journal.log`).
- Frontend : aucun fichier touché (AC-5), ses contrôles n'ont pas tourné ; **recette visuelle** : aucun contrôle visible n'est modifié, pas de capture.

**Limites et écarts déclarés** : (1) une édition de test a d'abord été faite par un script Python, aussitôt annulée (`git checkout`) et
refaite par l'outil Edit, conformément à l'interdit ; seuls `mut.py` et `run-mut.sh` (mutations, hors dépôt) emploient un script ;
(2) la file des verrous de la voie base a retenu les trois commandes lourdes (ciblé, mutations, gates) de 25 à 45 minutes chacune ;
(3) un `pkill -f` malvenu a été tapé dans une commande et **refusé** par le garde-fou avant exécution : aucun processus tué, aucune
conséquence ; (4) M-B3 rougit six e2e de plus que la fiche ne l'annonce (tableau ci-dessus) ; (5) `kesh_e2e_e50solde` a été créée et
laissée vide (aucun droit à retirer : `kesh%`.* est déjà accordé par `01-dev-grants.sql`).

**Statut à poser dans `sprint-status.yaml`** (par l'orchestrateur) : `50-solde-dette-epic: review`.

### File List

- `crates/kesh-db/src/repositories/supplier_invoices.rs` — `DuplicateRule::lifted_by_confirmation`, appel dans `guard_duplicates_in_tx`, doc-comments, quatre tests AC-1.
- `crates/kesh-db/src/errors.rs` — doc-comments (`DuplicateGesture`, `SupplierInvoiceDuplicate`, `SupplierInvoiceProbableDuplicate`).
- `crates/kesh-api/src/errors.rs` — `supplier_invoice_duplicate_response(existing, gesture, rule)`, les deux bras, doc-comments, quatre tests AC-2.
- `crates/kesh-api/src/routes/supplier_invoices.rs` — commentaires (D-B3).
- `_bmad-output/implementation-artifacts/50-solde-dette-epic.md` (cette fiche), `50-solde-dette-epic-review-prompt-p1.md`.

## Change Log

- **2026-11-24 — fiche créée** (Opus 5.5, `bmad-create-story`, en autonomie), à `48a66ca24` de `origin/epic/50-corrections`, sur trois
  issues fixées par l'orchestrateur (#1404, #1492, #1414) en trois lots ; sept modules, signal de splitting porté au Project Lead
  (commit `4978f345c`).
- **2026-11-24 — fiche réduite aux lots B et C** (Opus 5.5) : **décision du Project Lead du 2026-11-24**, le lot A (#1404) sort vers la
  v0.21.0 (route d'API neuve, quatre modules à lui seul, risque pour la coupure de jeudi). Retirés : D-A1 à D-A5, les AC, tests,
  mutations, tâches et l'inventaire du lot A, la note de version « Ajouté ». AC renumérotés : **10** (quatre du lot B, quatre du lot C,
  deux transverses). Modules : **quatre** (`kesh-db`, `kesh-api`, `kesh-i18n`, `shared`). Charge révisée : une demi-journée. Mutation
  M-B3 ajoutée (correspondance variante → règle dans `kesh-api`). Prémisse **à confirmer à l'étape 0** : la case (règle 2, dévalidation)
  de `details.confirmable` aux tests de l'API. Statut `drafted` ; `ready-for-dev` après la validation P1 (prompt
  `50-solde-dette-epic-validation-prompt-p1.md`, Sonnet).
- **2026-11-24 — validation P1 remédiée** (Opus 5.5, contexte de la rédaction). Validation P1 : Sonnet, contexte frais, **0 CRITICAL /
  0 HIGH / 1 MEDIUM / 11 LOW** (`kesh-gate-logs/e50solde/validation-p1.md`). Prémisse de chaque finding relue au code à `48a66ca24`
  (= tête d'`origin/epic/50-corrections`). **Aucun finding réfuté.** **V1-1** (MEDIUM) — prémisse vraie
  (`supplier_invoice_duplicate_e2e.rs:226` `for confirm in [false, true]`, `:231`, `:238`) : la preuve de M-B1 et M-B2 s'écrit
  assertion par assertion (tableau du § *Tests*) ; AC-2 en deux assertions nommées, relue et littérale ; sous M-B1, relue **verte**,
  littérale **rouge**, e2e `:231` vert et `:238` rouge à l'itération `confirm = false`. **V1-2** — prémisse vraie (`i18n.svelte.ts:15`) :
  « et AC-6 » retiré de M-C1, note ajoutée à l'AC-6. **V1-3** : AC-5 tenu par quatre `grep -cF`. **V1-4** — prémisse vraie
  (`routes/supplier_invoices.rs:349-353`, `:670`, `:749`, `:956` ; `kesh-db/src/errors.rs:581`) : sites ajoutés à l'inventaire et à
  D-B3, commande élargie (`inv-1492.txt`). **V1-5** — prémisse vraie (`supplier_invoice_duplicate.rs:1139-1141`) : M-B2 cite `:1141` ;
  « à confirmer » remplacé par le fait (aucun test HTTP de la case (règle 2, dévalidation)). **V1-6** : option retenue — le code se
  déduit de la règle dans la fonction (un seul `match`) ; « le compilateur lie » corrigé ; M-B3 redéfinie. **V1-7** : doc-comment de
  `api-health.svelte.ts` réécrit (`degradedOn503`, `fetch` directs). **V1-8** : regex élargie à « réessay- » (`inv-1414.txt`),
  `error-timeout`/`error-network` classés consignes hors classe. **V1-9** : AC-8 — assertion au scénario 1, scénario 3 exigé vert,
  `/health` à 503 pour la capture E6. **V1-10** : #1441-#1443 sourcées (décision de Guy du 2026-11-20, Q2, Change Log de la 48-11
  ligne 793) ; « comportement antérieur » de #1414 écrit ; #1509 citée. **V1-11** : italien à l'infinitif (« verificare »), comme
  `it-CH:804`, `:815`. **V1-12** : démarrage le mercredi 2026-11-25 au matin, ordre de sortie C puis B. AC : toujours **dix** ; modules :
  toujours **quatre** (`routes/supplier_invoices.rs` est dans `kesh-api`). Passe suivante : **P2, Opus, complète** (prompt
  `50-solde-dette-epic-validation-prompt-p2.md`).
- **2026-11-24 — validation P2 remédiée** (Opus 5.5, contexte de la rédaction). Validation P2 : **Opus**, contexte frais, **0 CRITICAL /
  0 HIGH / 1 MEDIUM / 6 LOW** (`kesh-gate-logs/e50solde/validation-p2.md`). Tendance : P1 Sonnet 0/0/1/11 → P2 Opus 0/0/1/6.
  **Signal de la règle de splitting préventif** : le MEDIUM **V2-1 recycle V1-1** (même preuve des mutations du lot B, défaut né de sa
  remédiation) ; V2-2 recycle V1-4, V2-5 recycle V1-7, V2-3 est né de la remédiation de V1-2. Signal **déclaré au Project Lead par
  l'orchestrateur** le 2026-11-24 ; **décision : la story reste entière**, le recyclage portant sur la rédaction des tests attendus, non
  sur le périmètre (quatre modules, sous la borne). Prémisse de chaque finding relue au code à `48a66ca24`. **V2-1** (MEDIUM) — prémisse
  vraie (`supplier_invoice_duplicate_e2e.rs:232` `assert_eq!(` ; `:238` n'est que le littéral) : `:238` → `:232` partout ; AC-1 et
  AC-2 en **un test par case**, nommés ; AC-2 dans l'ordre code, relue, littérale, texte ; § *Tests* réécrit pour que chaque attendu
  soit **observable** dans la sortie de cargo ou de vitest (l'assertion qui panique, `fichier:ligne` et message ; « passée » seulement
  par une assertion postérieure qui panique). **V2-2** — prémisse vraie (`kesh-db/src/errors.rs:577-578`, `supplier_invoices.rs:1873`,
  `:1566-1569`) : trois sites ajoutés à l'inventaire et à D-B3, `:1567` → `:1566-1569`, regex `-i` et « aucune sortie » rejouée
  (`inv-1492.txt`) ; récitations de contrat (`api-external.md`, `admin-manual.tex:2585`) écrites hors code, justes. **V2-3** — prémisse
  vraie (`i18n.svelte.ts:20` `const raw = _messages[key] || fallback;`) : `:15` → `:20`. **V2-4** — prémisse vraie
  (`api-client.ts:399-401` contre `:443-445`) : doc-comment inventorié et corrigé (D-C1, module `shared`). **V2-5** — prémisse vraie
  (`api-client.ts:101`, `:142`) : le doc-comment proposé dit « `fetchWithRetry` » et nomme les deux `fetch` directs. **V2-6** —
  prémisse **en partie fausse**, relue au code : le scénario 2 ne lit pas non plus le catalogue (`page.goto('/contacts')`, `:90`,
  recharge l'application, et `/api/v1/i18n/messages` est coupé par `:87`) ; remède retenu : assertion portée au scénario 2 **et**
  interception `:87` qui laisse passer `/api/v1/i18n/messages`, avec la mutation M-C3 qui le prouve. **V2-7** : phrase des passes
  ciblées réécrite (deux modules par lot ; gate complet pour le lot B). Greps de propagation : `:238`, `:1567`, `i18n.svelte.ts:15`,
  « redeviennent possibles », « vertes / rouges » sur la fiche — ne restent qu'aux entrées historiques du Change Log. AC : **dix** ;
  modules : **quatre**. Passe suivante : **P3, Sonnet, complète** (prompt `50-solde-dette-epic-validation-prompt-p3.md`).
- **2026-11-24 — validation P3 ; lot C sorti ; validation terminée, `ready-for-dev`** (Opus 5.5, contexte de la rédaction). Validation
  P3 : **Sonnet**, contexte frais, **0 CRITICAL / 0 HIGH / 2 MEDIUM / 3 LOW** (`kesh-gate-logs/e50solde/validation-p3.md`). Tendance :
  P1 Sonnet 0/0/1/11 → P2 Opus 0/0/1/6 → P3 Sonnet 0/0/2/3. **Les deux MEDIUM sont du lot C et recyclent** : V3-1 → V1-2 (la garde
  `i18n-repli-divergent-actif` ne garde pas `db-unavailable-banner` ; M-C1 survivait, M-C2 ne rougissait pas la garde), V3-2 → V2-6, né
  de sa remédiation (le scénario 2 ne lit pas le catalogue ; M-C3 ne pouvait pas échouer) ; V3-5 (LOW, lot C) recycle V1-2 ; V3-4 (LOW,
  lot C) est neuf. Troisième recyclage de suite sur la preuve du lot C : signal de la règle de splitting préventif, déclaré au Project
  Lead par l'orchestrateur. **Décision du Project Lead du 2026-11-24 : le lot C (#1414) sort de la story vers la v0.21.0** (recyclage de
  la preuve à trois passes, P4 cosmétique, coupure de jeudi) ; piste simplifiée écrite au § *Périmètre* (G13 `REPLIS_A_SITE_UNIQUE`,
  sans E2E du catalogue, reprendre V3-1, V3-2, V3-4, V3-5) ; étude du lot C conservée au commit `df32a86dc`. Retirés : D-C1, les AC-5 à
  AC-8 du lot C, M-C1 à M-C3, l'inventaire du lot C, les captures, la note de version « Modifié » ; AC renumérotés sans trou (**six** :
  AC-1 à AC-4 inchangés, ancien AC-9 → AC-5, ancien AC-10 → AC-6) ; modules : **deux** (`kesh-db`, `kesh-api`) ; charge : deux à trois
  heures de développement. **Lot B** : aucun finding au-dessus de LOW à la P3 ; **V3-3** (LOW) — prémisse vraie
  (`kesh-db/tests/supplier_invoice_duplicate.rs:206` `rule_1_same_number_same_year_is_refused_without_exit`, cas `CONFIRMED` `:222` ;
  `:674` `update_replays_the_rule_only_when_one_of_the_four_changes`, `:762` `for policy in [REFUSE, CONFIRMED]`) : les deux tests
  ajoutés au tableau de M-B1 comme échecs attendus hors discrimination, et les noms de tests écrits comme suffixes du chemin complet
  que nextest imprime. Remédiation **textuelle et locale** (une ligne de tableau, une phrase), sans changement de décision ni d'AC.
  **Validation terminée** : la dernière passe ne rend aucun finding au-dessus de LOW pour ce qui reste dans la story (`CLAUDE.md`
  § *Review Iteration Rule*) ; **`Status: ready-for-dev`**. Statut à poser par l'orchestrateur dans `sprint-status.yaml` :
  `50-solde-dette-epic: ready-for-dev`.
- **2026-11-24 — développement** (Sonnet 5.5, `bmad-dev-story`, en autonomie) : rouge `cf8c2c01a` (huit tests, non-compilation E0599),
  code `88905d0ff` (D-B1, D-B2, D-B3), mutations M-B1 à M-B3 rejouées (assertions comparées à l'attendu, Dev Agent Record ; écart : M-B3
  rougit six e2e de plus que la fiche), gate complet vert (`test-fast.sh --ci` 6270/6270, `gate-ci.sh` 6289 passés, 0 échoué).
  Aucune décision de la fiche modifiée, aucun test existant touché. **`Status: review`** ; revue de code P1 non lancée (prompt :
  `50-solde-dette-epic-review-prompt-p1.md`). E2E complet à la clôture.
