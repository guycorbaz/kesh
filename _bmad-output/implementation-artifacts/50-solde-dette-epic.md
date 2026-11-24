# Story 50-solde : Solde de la dette de l'Epic 50 — une seule règle de levée d'un refus de doublon (#1492), un bandeau « base indisponible » qui ne promet pas de réessai (#1414)

Status: drafted — fiche écrite le 2026-11-24, réduite le même jour aux lots B et C par décision du Project Lead ; validation P1 (Sonnet, 0/0/1/11) remédiée ; **validation P2 à lancer** (prompt `50-solde-dette-epic-validation-prompt-p2.md`) ; passe à `ready-for-dev` **après** la validation seulement (consigne de l'orchestrateur).

*Fiche rédigée le 2026-11-24 (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-50-solde`, branche `story/50-solde-dette-epic`, partie de
`origin/epic/50-corrections` à **`48a66ca24`**. **Tout le code cité est lu à `48a66ca24`.** Release visée : **0.20.0**, vendredi 2026-11-27
(coupure le jeudi 2026-11-26 au soir). **Aucune migration.** Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e50solde/` (inventaires
`inv-1492.txt`, `inv-1414.txt`, régénérables par les commandes qu'ils portent en tête). Clé courte des bases : `e50s`
(`kesh_e50s`, `kesh_ci_e50s`, `kesh_e2e_e50s`).*

## Story

En tant que comptable qui enregistre ses factures fournisseur et travaille parfois sur une base qui flanche,
je veux que la règle « ce refus de doublon se lève-t-il par une confirmation ? » n'ait qu'une source, et que le bandeau « base
indisponible » ne me promette pas un réessai qui n'aura pas lieu pour mon enregistrement,
afin que le serveur ne puisse pas dire « confirmable » d'un refus qu'il maintient, et que je vérifie mon opération au lieu de croire
qu'elle sera rejouée.

## Périmètre : la dette des revues de l'Epic 50

Story de solde (`CLAUDE.md` § *Priorités des défauts* : « seulement la dette des stories de l'epic, plafonnée par un nombre d'issues
fixé au kickoff ») ; le fichier d'épique dit « plafond fixé à la reprise » : **trois issues**, fixé par l'orchestrateur le 2026-11-24,
choisies **par priorité, puis par ancienneté**, parmi la dette ouverte par les revues des stories 48-11, 48-12a et 48-12b. La première
rédaction de la fiche portait les trois en trois lots indépendants (A, B, C) et touchait **sept modules**, au-dessus de la borne de cinq
de la règle de splitting ; le signal a été porté au Project Lead.

**Décision du Project Lead du 2026-11-24** : la story ne garde que les **lots B (#1492) et C (#1414)** ; le **lot A (#1404) sort vers la
release v0.21.0** (l'orchestrateur replanifie l'issue). Raisons : il exigeait une **route d'API neuve**
(`GET /api/v1/imported-supplier-invoices/{id}`, un ajout au contrat, avec sa documentation dans le guide de l'API et le manuel
d'administration), il touchait à lui seul **quatre modules** (`kesh-api`, `features/imported-supplier-invoices`, la route
`supplier-invoices`, la documentation) — ce qui interdit les passes ciblées de la boucle de revue —, et il portait l'essentiel de la
charge, donc **le risque pour la coupure du jeudi 2026-11-26 au soir**. Les lots B et C restent sous leur lettre d'origine.

| Issue | Labels | Origine | Sort |
|---|---|---|---|
| **#1492** | `technical-debt`, P3 | revue de code P3 de la 48-12a (P3-5) | **retenue** (lot B) |
| **#1414** | `bug`, P4 | revue P1 de la 48-12b (lentilles A et E), créée le 2026-11-19 | **retenue** (lot C) — la plus ancienne des P4 (#1415 et #1416, du même jour, ont un numéro postérieur). L'issue le dit elle-même : « **Comportement antérieur à la story** » ; elle reste de la dette **de revue** parce que c'est la revue de la 48-12b qui l'a relevée et ouverte, comme #1415 (« pas une régression ») |
| #1404 | `technical-debt`, P3 | validation P2 de la 48-12b (V2-9) | **sortie vers la v0.21.0** (décision du Project Lead ci-dessus) |
| #1441, #1442, #1443 | `bug`, P2 / P4 / P3 | arrondis au pair relevés à la **rédaction** de la fiche 48-11 | **écartées** : sorties du périmètre de la 48-11 par **décision de Guy du 2026-11-20** (fiche `48-11-arrondi-au-centime.md`, Change Log, ligne 793 : « **Q2** et issues : pain.001 → **#1441**, I1 → **#1442**, I2 → **#1443** » ; aussi Dev Notes, ligne 590 : « Issues liées (décisions de Guy du 2026-11-20, Q2 — ouvertes, hors story) ») ; ce ne sont pas des constats de revue, mais des défauts voisins écartés au cadrage : le critère « priorité puis ancienneté » ne s'applique qu'à la dette de revue, d'où #1441 (P2) laissée de côté |
| #1415, #1416 | `bug`, P4 | revue de la 48-12a (formes Unicode du numéro) | **écartées** : P4, sous le plafond |
| #1453 | `documentation`, P4 | revue de la 48-12b (doc-comment de `unknown-outcome.ts`) | **écartée** : P4, sous le plafond |
| #1491 | `bug`, P4 | revue de la 48-12a (libellé d'audit) | **écartée** : P4, sous le plafond |

Les deux lots sont **indépendants** : aucun ne lit le code de l'autre ; ils se développent, se testent et se relisent séparément.

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
   `assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DUPLICATE"), "{e}");`, `:238` (`"confirmable": false`, règle 1 à la saisie) ;
   `:263` (règle 2 à la saisie, `true`), `:421` (règle 1 à la dévalidation, `true`). **Fait établi** (validation P1, V1-5,
   `grep -n confirmable` → `:4, :212, :238, :263, :375, :421` ; `grep -nF PROBABLE` → `:260` seul) : **la case (règle 2, dévalidation)
   n'est assertée par aucun test HTTP** ; l'AC-2 est seule à la tenir côté `details.confirmable`. Un module de tests unitaires existe dans `kesh-api/src/errors.rs`
   (`:6587`), avec l'aide `response_body(resp: Response) -> (StatusCode, serde_json::Value)` (`:6605`) ; un autre dans
   `supplier_invoices.rs` (`:3838`).

**Inventaire par le symptôme** (« une décision sur la levée d'un refus de doublon », `inv-1492.txt`) :
`git grep -nE "liftable|confirmable|DuplicateGesture::Unvalidation|probable \|\|" -- 'crates/*/src/*'`,
`git grep -nE "confirmable" -- frontend/src ':!*.test.ts'` et, pour les récitations écrites sans ces mots (validation P1, V1-4),
`git grep -nE "jamais.{0,40}(règle 1|SUPPLIER_INVOICE_DUPLICATE)|lève.{0,30}(deux règles|règle 2)|les deux règles|levé \(règle" -- 'crates/*/src/*'`
(sortie ajoutée à `inv-1492.txt`). **Sites qui décident** : deux, `supplier_invoices.rs:658-661` et
`errors.rs:6260` — tous deux corrigés (D-B1, D-B2). **Sites qui lisent ou décrivent** : `supplier_invoices.rs:1883` (passe le geste,
ne décide pas) ; doc-comments et commentaires qui récitent la table — `kesh-db/src/errors.rs:574`, `:580-581`, `:1892-1894`, `:1905`,
`supplier_invoices.rs:449-462`, `:1567`, `kesh-api/src/errors.rs:5578-5581`, `:6240`, `:6247-6251`, `kesh-api/src/routes/supplier_invoices.rs:349-353`
(« il ne lève **jamais** `SUPPLIER_INVOICE_DUPLICATE` »), `:669-673` (« les deux règles »), `:749-750` (« levé (règle 2 seulement) »),
`:956-957` — réécrits pour **nommer la source** (D-B3) ;
frontend (`supplier-invoice-form.ts`, `SupplierInvoiceForm.svelte:1193`, `[id]/+page.svelte:1016`) : lit le booléen, inchangé. Les
autres « confirmable » du relevé (`kesh-import`, `payment_batches.rs:351`, `errors.rs:1073`, `:1096`, `BankImportUpload.svelte:488`)
parlent d'autres refus : hors classe.

### Lot C — #1414 : le bandeau promet un réessai qui n'a pas lieu pour une écriture

1. **Le texte** : `crates/kesh-i18n/locales/fr-CH/messages.ftl:88`
   `db-unavailable-banner = Base de données temporairement indisponible — réessai automatique en cours`, et ses trois traductions à la
   même ligne (`de-CH` « automatischer Wiederholungsversuch läuft », `en-CH` « retrying automatically », `it-CH` « nuovo tentativo
   automatico in corso ») ; le repli Svelte, identique au français : `frontend/src/lib/shared/components/DegradedBanner.svelte:14-17`.
2. **Qui allume le bandeau** : `frontend/src/lib/shared/utils/api-client.ts:420` (`if (call.degradedOn503 !== false)
   apiHealth.setDegraded();`, réponse 503, **quelle que soit la méthode**) et `:448` (`apiHealth.setDegraded();` dans le `catch` d'une
   coupure ou d'un délai, **quelle que soit la méthode**) ; aussi `routes/+layout.svelte:94`, `:97` et
   `lib/app/stores/auth.svelte.ts:411`, `:425` (chargement initial).
3. **Qui est rejoué** : seules les méthodes idempotentes (`api-client.ts:321-322`, `isIdempotentMethod` : `undefined`, `GET`, `HEAD`) ;
   `maxAttempts` vaut 1 pour une écriture (`:414`). Un `POST` ou un `PUT` n'est **jamais** renvoyé : sur le formulaire figé en E6 (48-12b),
   le bandeau contredit l'avis, qui dit que rien n'a été renvoyé.
4. **Ce qui est vrai pendant l'état dégradé** : `api-health.svelte.ts` interroge `GET /health` toutes les 5 s et éteint le bandeau à la
   première réponse `{ db: true }` (`pollHealth`) ; tout `2xx` l'éteint aussi (`api-client.ts:443-445`). Le bandeau disparaît donc **de
   lui-même** quand la base répond.
5. **Le doc-comment qui porte la croyance fausse** : `api-health.svelte.ts:4-6` — « quand l'API est injoignable ou que la DB est down
   (réponses NETWORK_ERROR / TIMEOUT / 503 sur les requêtes idempotentes traversant `api-client.ts`) » : faux, le `catch` et le 503
   allumant le bandeau pour toute méthode (point 2).
6. **Les tests** : aucun test unitaire de `DegradedBanner.svelte` (`git grep -ln "degraded-banner" frontend/src` ne rend que le
   composant) ; l'E2E `frontend/tests/e2e/db-resilience.spec.ts:59-61` vérifie
   `toContainText(/Base de données temporairement indisponible/)` — le **début** du texte, qui ne change pas. La garde
   `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts` rougit si le repli Svelte diverge de la valeur `fr-CH` d'une clé présente
   au catalogue : elle tient l'alignement du repli et du catalogue.

**Inventaire par le symptôme** (« un texte qui promet un réessai », `inv-1414.txt`) :
`git grep -niE "réessai automatique|retrying automatically|Wiederholungsversuch|tentativo automatico" -- crates frontend docs/manual`
→ les quatre catalogues `:88`, `DegradedBanner.svelte:16` et un commentaire d'E2E (`supplier-invoice-unknown-outcome.spec.ts:161`,
« bandeau « réessai automatique » ») ; `grep -niE "réessai|réessay|rejou|reconnex|tentative|automatiquement|reprend" crates/kesh-i18n/locales/fr-CH/messages.ftl`
(regex élargie à « réessay- » après la validation P1, V1-8 ; sortie ajoutée à `inv-1414.txt`) → la ligne 88 et des **consignes à
l'utilisateur** (« Réessayez… »), dont `error-timeout` (`fr-CH:2633`, « Le serveur ne répond pas. Réessayez ultérieurement. ») et
`error-network` (`:2632`), et des libellés sans rapport (`:4018` « Copié automatiquement par Kesh », `:4026` « Reprendre la valeur corrigée ») : aucun ne promet un réessai **automatique** — hors classe. Les replis Svelte (`frontend/src` hors
tests) et les messages Rust (`'crates/*/src/*'`) n'en contiennent pas d'autre. **Ensemble clos** : la clé et son repli (corrigés, D-C1),
le commentaire d'E2E (réécrit, D-C1). Hors code produit :
`CHANGELOG.md:1711` et `_bmad-output/planning-artifacts/epic-10.md:207` citent l'ancien texte — **historique, inchangé**.

### Manuels

- **Lot B** : rien de visible ; l'admin-manual (`docs/manual/fr/admin-manual.tex:2585`, section `sec:maj-0-20-0`) décrit `confirmable`
  (« jamais le premier à la création, au \texttt{PUT} et à la complétion …, toujours le second, et les deux au repassage d'une
  annulée ») : **reste vrai**, aucune réécriture.
- **Lot C** : aucun manuel ne cite le bandeau (`grep` des quatre formulations sur les `.tex` et sur les PDF aplatis par
  `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` : vide ; `ls docs/manual/*/` : seul `fr` porte des `.tex`). Rien à écrire.
- **Aucun manuel ne change** dans cette story ; la validation le recontrôle (axe A8 du prompt).

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
- **D-B3 — les doc-comments nomment la source** : ceux qui récitent la table (`kesh-db/src/errors.rs:574`, `:580-581`, `:1892-1894`,
  `:1905` ; `supplier_invoices.rs:449-462`, `:1567` ; `kesh-api/src/errors.rs:5578-5581`, `:6240`, `:6247-6251` ;
  `kesh-api/src/routes/supplier_invoices.rs:349-353`, `:669-673`, `:749-750`, `:956-957`) gardent leur résumé et renvoient à
  `DuplicateRule::lifted_by_confirmation` — sans en changer le sens. `routes/supplier_invoices.rs` est dans `kesh-api`, module déjà
  compté.

### Lot C — #1414

- **D-C1 — un texte qui ne promet rien, sans changer la mécanique** : la clé `db-unavailable-banner` (même clé, même `data-testid`)
  prend, dans les quatre catalogues et dans le repli de `DegradedBanner.svelte` (identique au `fr-CH`, garde
  `i18n-repli-divergent-actif`) :
  - `fr-CH` : « Base de données temporairement indisponible — ce bandeau disparaît dès qu’elle répond de nouveau. Avant de recommencer
    une opération, vérifiez si elle a été enregistrée. »
  - `de-CH` : « Datenbank vorübergehend nicht verfügbar — dieser Hinweis verschwindet, sobald sie wieder antwortet. Prüfen Sie vor dem
    Wiederholen eines Vorgangs, ob er gespeichert wurde. »
  - `en-CH` : « Database temporarily unavailable — this banner disappears as soon as it responds again. Before repeating an operation,
    check whether it was saved. »
  - `it-CH` : « Database temporaneamente non disponibile — questo avviso scompare non appena risponde di nuovo. Prima di ripetere
    un’operazione, verificare se è stata registrata. »
  Chaque phrase est vraie pour une lecture comme pour une écriture (point 4 du lot C : le bandeau s'éteint de lui-même) ; aucune ne dit
  « réessai ». Registre : `Sie` en allemand, comme partout (`de-CH:2501`, `:804`) ; en italien, **l'infinitif**, celui des clés les plus
  proches par le sens — l'issue inconnue d'une écriture : `it-CH:804` `invoice-settle-unknown-outcome` (« Verificare gli importi … »),
  `:815` `mcp-sends-error-unknown-outcome` (« Ricaricare per saperlo. ») — plutôt que le `tu` d'`error-network` (`it-CH:2501`,
  « Verifica la connessione ») (validation P1, V1-11) ;
  apostrophe typographique U+2019 dans `fr-CH` et `it-CH`, comme les clés récentes. Le doc-comment `api-health.svelte.ts:4-6` se
  corrige (V1-7) : « allumé par une coupure, un délai ou un 503 sur toute requête de `api-client.ts`, quelle que soit sa méthode (sauf
  le 503 d'un appel `degradedOn503: false`, `exchange-rates.api.ts:37`), et par l'échec du `/health` ou de `/me` du chargement initial
  (`routes/+layout.svelte:94`, `:97` ; `auth.svelte.ts:411`, `:425`, `fetch` direct) ; seules les méthodes idempotentes sont rejouées » ; le commentaire d'E2E
  `supplier-invoice-unknown-outcome.spec.ts:161` cesse de citer « réessai automatique ».
  **Options écartées** : ne plus allumer le bandeau pour une écriture (masquerait une panne réelle, et le bandeau suivant viendrait du
  premier `GET`) ; deux textes selon la méthode (l'état `apiHealth` est **global** et ne sait pas quelle requête l'a allumé : un `GET`
  concurrent le rallume) ; le texte proposé par l'issue (« … — vérifiez l'état de votre dernière opération avant de recommencer ») —
  repris en substance, avec la phrase qui dit au lecteur que le bandeau s'en ira seul (ce que l'ancien « réessai » cherchait à dire).

## Critères d'acceptation

### Lot B — #1492

1. **AC-1 — la table, une fois** (`supplier_invoices.rs`, module `tests`) : un test pur sur les **quatre** cases de
   `DuplicateRule::lifted_by_confirmation` — (règle 1, `Entry`) → `false`, (règle 2, `Entry`) → `true`, (règle 1, `Unvalidation`) →
   `true`, (règle 2, `Unvalidation`) → `true`.
2. **AC-2 — la réponse relit la table** (`kesh-api/src/errors.rs`, module `tests`, avec `response_body`) : pour chacune des quatre
   cases, `AppError::Database(DbError::Supplier…Duplicate { existing, gesture })` rend `409`, le bon `error.code`, et
   `details.confirmable` comparé **par deux assertions distinctes, chacune avec un message qui nomme la case et sa nature** : l'assertion
   **relue** (`== DuplicateRule::<règle>.lifted_by_confirmation(gesture)`, message « relue (règle N, geste) ») et l'assertion **littérale**
   (`== <true|false>`, message « littérale (règle N, geste) ») — l'une prouve le lien, l'autre la valeur, et c'est **laquelle rougit**
   sous M-B1/M-B2 qui distingue une table de deux (§ *Tests*) ; le message reste celui d'aujourd'hui pour chaque case (clés
   `error-supplier-invoice-duplicate`, `-probable-duplicate`, `-duplicate-unvalidate`).
3. **AC-3 — plus aucune seconde table** :
   `git grep -nE "let liftable|probable \|\||gesture == DuplicateGesture::Unvalidation" -- 'crates/*/src/*'` est **vide** ;
   `git grep -nw lifted_by_confirmation -- 'crates/*/src/*'` ne rend que la définition, son appel dans `guard_duplicates_in_tx`, son
   appel dans `supplier_invoice_duplicate_response`, les doc-comments de D-B3 et les tests. Sortie copiée au Dev Agent Record.
4. **AC-4 — rien ne change au comportement** : `crates/kesh-db/tests/supplier_invoice_duplicate.rs`,
   `supplier_invoice_duplicate_lock.rs` et `crates/kesh-api/tests/supplier_invoice_duplicate_e2e.rs` passent **sans modification**
   (`git diff origin/epic/50-corrections...HEAD --stat -- <ces trois fichiers>` vide). Aucun E2E neuf : rien ne change à la frontière
   HTTP ni dans le navigateur ; les e2e existants de l'API (`:238`, `:263`, `:421`) jouent déjà la réponse.

### Lot C — #1414

5. **AC-5 — le texte** : les quatre catalogues portent, à la clé `db-unavailable-banner`, les textes de D-C1, mot pour mot — quatre
   commandes, une par locale, `grep -cF "db-unavailable-banner = <texte exact de D-C1>" crates/kesh-i18n/locales/<l>-CH/messages.ftl`
   rendent chacune **1** (sorties copiées) ; le repli de `DegradedBanner.svelte` égale la valeur `fr-CH` ; `i18n-repli-divergent-actif.test.ts` et `npm run lint-i18n-ownership` verts.
6. **AC-6 — le composant** (`frontend/src/lib/shared/components/DegradedBanner.test.ts`, neuf) : après `apiHealth.setDegraded()`,
   `degraded-banner` affiche le texte français de D-C1 ; il ne contient ni « réessai » ni « automatique » ; après
   `apiHealth.clearDegraded()`, il disparaît (filet, déjà vrai). État global remis entre les tests (`clearDegraded()` en `afterEach`).
   Ce test lit le **repli Svelte** (`i18n.svelte.ts:15`, `_messages[key] || fallback`, catalogue vide en test ; même patron que
   `SessionIdleWarning.test.ts:8`) : le catalogue `fr-CH` est tenu par la garde `i18n-repli-divergent-actif` et par l'AC-5.
7. **AC-7 — plus aucune promesse** :
   `git grep -niE "réessai automatique|retrying automatically|Wiederholungsversuch läuft|tentativo automatico" -- crates frontend docs/manual`
   est **vide**. Sortie copiée.
8. **AC-8 — de bout en bout et recette visuelle** : `db-resilience.spec.ts`, **trois scénarios verts** — le scénario 1 (`:31`) reçoit,
   après l'assertion existante `:61`, l'assertion ajoutée `await expect(banner).toContainText('vérifiez si elle a été enregistrée');` ;
   le scénario 2 (`:68`) inchangé ; le scénario 3 (`:96`, « banner disparaît automatiquement ») inchangé et **vert**, puisqu'il tient la
   promesse neuve « ce bandeau disparaît dès qu’elle répond de nouveau ». **Captures** (règle *Recette visuelle* du `CLAUDE.md`, thème
   clair, base de démonstration) du bandeau seul (scénario 1) et du bandeau **avec** l'avis E6 de la complétion (montage de la capture
   `e4812b-E6-completion` de la 48-12b, `supplier-invoice-duplicate.spec.ts:517-540`, **complété d'un `page.route('/health', …)` qui rend
   503**, comme le scénario 3 `:117-120`, pour que le ping de 5 s n'éteigne pas le bandeau avant la capture), sous
   `kesh-gate-logs/e50solde/captures/`, jointes au Dev Agent Record et au prompt de la revue de code.

### Transverses

9. **AC-9 — aucune facture, aucun avoir, aucune écriture ne change** : aucun fichier de `crates/kesh-qrbill`, `crates/kesh-report`,
   `crates/kesh-core`, ni `crates/kesh-db/migrations/` n'est touché (`git diff origin/epic/50-corrections...HEAD --stat --
   crates/kesh-qrbill crates/kesh-report crates/kesh-core crates/kesh-db/migrations` vide) ; les seuls changements de `kesh-db` sont ceux
   de D-B1/D-B3, sans SQL modifié (`git diff origin/epic/50-corrections...HEAD -- crates/kesh-db | grep -E '^[+-].*(SELECT|UPDATE|INSERT|DELETE)'`
   vide) ; l'audit `supplier_invoice.duplicate_confirmed` s'écrit dans les mêmes cas qu'avant (AC-4).
10. **AC-10 — gates** (§ *Gates*) verts au dernier commit de code ; E2E complet « vert » au sens du `CLAUDE.md` (seuls les échecs
    attendus, jugés fichier par fichier).

## Tests à écrire, et la preuve du rouge

Journaux attendus sous `/home/gcorbaz/devel/kesh-gate-logs/e50solde/dev/`.

| Test | Rouge sans le correctif | Preuve |
|---|---|---|
| AC-1 | — (la méthode est neuve : le test ne compile pas avant elle) | **M-B1** et **M-B2**, ci-dessous : AC-1 rouge sous chacune |
| AC-2 | ne rougit pas sur le code d'avant (les valeurs sont les mêmes) : **filet**, et c'est sa raison d'être — il rougit sous M-B1/M-B2/M-B3 | déclaré |
| AC-5, AC-6 | **rouge d'abord** : AC-6 rougit sur l'ancien texte ; **M-C1** : l'ancien texte remis dans le seul `fr-CH` → `i18n-repli-divergent-actif` rouge (AC-6, qui lit le repli Svelte, reste **vert** : V1-2) (`mut-c1.log`) ; **M-C2** : l'ancien texte remis dans le seul repli Svelte → `i18n-repli-divergent-actif` rouge (`mut-c2.log`) | `rouge-front.log` |
| AC-8 | rouge d'abord pour l'assertion ajoutée | `rouge-e2e.log` (`db-resilience.spec.ts`) |
| AC-3, AC-4, AC-7, AC-9 | — | commandes et sorties copiées |

**Les mutations du lot B, assertion par assertion** (validation P1, V1-1). Une mutation qui ne touche **que** le corps de
`lifted_by_confirmation` ne distingue « une table » de « deux tables » que par **l'assertion qui rougit** : chaque journal `mut-b*.log`
**nomme** les assertions rouges et vertes (messages d'AC-2, ligne et itération de l'e2e), et la preuve est l'ensemble ci-dessous,
tout entier. Sous l'hypothèse contraire (`kesh-api` garderait sa table), les colonnes « relue » et « littérale » s'inverseraient et l'e2e
rougirait à `:231` sur l'itération `confirm = true` (`201` au lieu de `409`) : c'est ce qu'il faut **ne pas** voir.

| Mutation (seul fichier touché : `supplier_invoices.rs`, corps de `lifted_by_confirmation`) | Assertion | Attendu |
|---|---|---|
| **M-B1** : (règle 1, `Entry`) → `true` (`mut-b1.log`) | AC-1, case (règle 1, `Entry`) | **rouge** |
| | AC-2, « relue (règle 1, Entry) » | **verte** — `kesh-api` lit la table mutée |
| | AC-2, « littérale (règle 1, Entry) » (`false`) | **rouge** |
| | e2e `create_twice_is_refused_with_details_and_message`, itération `confirm = false` : `:231` | **verte** (le refus reste `409 SUPPLIER_INVOICE_DUPLICATE` sous `Refuse`) |
| | même itération : `:238` (`"confirmable": false`) | **rouge** (le test s'arrête là ; l'itération `confirm = true` n'est pas atteinte) |
| **M-B2** : (règle 2, `Unvalidation`) → `false` (`mut-b2.log`) | AC-1, case (règle 2, `Unvalidation`) | **rouge** |
| | AC-2, « relue (règle 2, Unvalidation) » | **verte** |
| | AC-2, « littérale (règle 2, Unvalidation) » (`true`) | **rouge** |
| | `kesh-db/tests/supplier_invoice_duplicate.rs:1136` (refus sous `REFUSE`) | **verte** |
| | même test, `:1139-1141` (`repasser(…, CONFIRMED).await.expect("confirmé")`) | **rouge** (le refus n'est plus levé) |
| **M-B3** : dans `kesh-api/src/errors.rs`, les règles des deux bras `DbError::SupplierInvoiceDuplicate` / `…ProbableDuplicate` échangées (`mut-b3.log`) | AC-2, `error.code` des quatre cases | **rouge** (le code se déduit de la règle, D-B2) |

Le test AC-2 doit écrire ses deux assertions **séparément** (pas un seul `assert_eq!` sur un tuple), sans quoi le journal ne dit pas
laquelle rougit. Après chaque mutation restaurée, **toucher le fichier** avant de relancer (cargo peut garder le binaire muté).

## Tâches

1. **Étape 0** : relire à la tête de la branche les lignes citées (`supplier_invoices.rs:449-460`, `:484-502`, `:622-674`, `:1567`,
   `:1875-1886` ; `kesh-db/src/errors.rs:570-585`, `:1885-1912`, `:2606-2607` ; `kesh-api/src/errors.rs:5576-5598`, `:6230-6290`,
   `:6587-6620` ; `messages.ftl:88` des quatre locales ; `DegradedBanner.svelte` ; `api-health.svelte.ts:1-15` ; `api-client.ts:410-450`) ;
   en copier trois au Change Log ; régénérer les deux inventaires (commandes en tête de `inv-1492.txt` et `inv-1414.txt`) et noter tout
   écart (une reprise de `main` dans `epic/50-corrections` est en cours : un site neuf entre au relevé avant tout code). Relire aussi
   `routes/supplier_invoices.rs:349-353`, `:669-673`, `:749-750`, `:956-957` (D-B3) et `db-resilience.spec.ts:31-140` (AC-8).
2. Tests rouges (AC-6, AC-8), journaux `rouge-*.log`.
3. **Lot B** : `lifted_by_confirmation`, appel dans `guard_duplicates_in_tx` ; `kesh-api` (D-B2) ; doc-comments (D-B3) ; tests AC-1,
   AC-2 ; mutations M-B1 à M-B3 ; **gate complet** au commit qui porte ce patch (exception `kesh-db` du `CLAUDE.md` : il touche un
   repository).
4. **Lot C** : quatre catalogues, repli, doc-comment `api-health.svelte.ts`, commentaire d'E2E ; test AC-6 ; E2E AC-8 ; M-C1, M-C2 ;
   captures.
5. Propagation (grep du symptôme de chaque lot : AC-3, AC-7), AC-4, AC-9 ; Dev Agent Record. **Ne pas écrire `CHANGELOG.md` ni
   `sprint-status.yaml`** (l'orchestrateur tient les statuts).

Un commit par étape franchie (test rouge, correctif vert, remédiation), poussé sur `story/50-solde-dette-epic` seule (`CLAUDE.md`
§ *Worktrees et agents*), chacun avec `refs #1492` ou `refs #1414` selon le lot ; **aucun mot-clé de fermeture**.

## Gates

- **En cours de développement** (gate ciblé) : `cargo fmt --all -- --check` ;
  `cargo clippy --workspace --all-targets -- -D warnings` ;
  `cargo nextest run -E 'binary(supplier_invoice_duplicate_e2e) | binary(supplier_invoice_duplicate) | binary(supplier_invoice_duplicate_lock) | (package(kesh-api) & kind(lib)) | (package(kesh-db) & kind(lib))'`
  sur `kesh_e50s` ; frontend : `npm run check`, `npm run lint-i18n-ownership`, `npx vitest run` sur les fichiers touchés et
  `i18n-repli-divergent-actif.test.ts`. **Exception `kesh-db`** : le lot B touche un repository → **gate complet** au commit qui le
  porte, même en cours de boucle.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh` (base `kesh_ci_e50s`) ; frontend complet
  (`npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`).
- **À la clôture**, au dernier commit de code : `gate-ci` et **E2E complet** (`docs/testing.md` § *Prérequis Playwright local*,
  `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés, base `kesh_e2e_e50s` ; « vert » = les seuls échecs attendus, jugés
  fichier par fichier).
- Une commande lourde à la fois sous `scripts/mem-guard.sh`, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant, verrou
  `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`. Le story file n'affirme que ce qui a tourné.

## Note de version

**Modifié**

- Le bandeau « Base de données temporairement indisponible » ne promet plus un « réessai automatique » : il dit qu'il disparaîtra quand
  la base répondra, et invite à vérifier si la dernière opération a été enregistrée avant de la recommencer — une saisie enregistrée
  n'est jamais renvoyée d'elle-même (refs #1414).

**Interne**

- La règle qui dit si une confirmation lève un refus de doublon de facture fournisseur n'est plus écrite qu'une fois (refs #1492).

## Modules — règle de splitting

Grain de la 48-12b (une crate backend, un domaine ou une route du frontend, ou la documentation ; les tests suivent leur module) :

| # | Module | Lot | Fichiers de production |
|---|---|---|---|
| 1 | `kesh-db` | B | `repositories/supplier_invoices.rs`, `errors.rs` (doc-comments) |
| 2 | `kesh-api` | B | `errors.rs`, `routes/supplier_invoices.rs` (commentaires, D-B3) |
| 3 | `kesh-i18n` | C | `locales/{fr,de,en,it}-CH/messages.ftl` |
| 4 | `shared` (frontend) | C | `components/DegradedBanner.svelte`, `utils/api-health.svelte.ts` (doc-comment) |

**Quatre modules : sous la borne de cinq, pas de découpage.** Le signal de la première rédaction (sept modules) est résolu par la
décision du Project Lead du 2026-11-24 (§ *Périmètre*). Aucune documentation ni aucun manuel ne change. Tests touchés hors de ces
fichiers : `DegradedBanner.test.ts` (neuf), `db-resilience.spec.ts` (une assertion), commentaire de
`supplier-invoice-unknown-outcome.spec.ts:161`.

## Dev Notes

- **Charge révisée** : lot B, une à deux heures, plus trois mutations (chacune recompile `kesh-db` puis `kesh-api`) et le gate complet
  qu'impose l'exception `kesh-db` ; lot C, une heure et demie avec les captures. **Une demi-journée** de développement, boucles de
  validation et de revue non comprises ; s'y ajoutent `gate-ci`, l'E2E complet et au moins deux passes de revue alternées. **Calendrier** :
  le développement **démarre le mercredi 2026-11-25 au matin** ; c'est la condition pour clore avant la coupure du jeudi 2026-11-26 au
  soir (V1-12). La remédiation d'une passe ne touche en principe qu'un lot, donc un ou deux modules : les passes ciblées redeviennent
  possibles. **Si le temps manque**, retirer un lot entier plutôt que le livrer à moitié : **C sort en premier** (P4), **puis B** ; la
  décision revient au Project Lead.
- **Fichiers communs** : avec la 48-12a (close, fusionnée dans `epic/50-corrections`) — `supplier_invoices.rs`, `kesh-api/src/errors.rs` ;
  catalogues i18n (chaque epic dans son bloc ; ici une valeur existante change, aucune clé neuve) ; `supplier-invoice-unknown-outcome.spec.ts`
  (un commentaire). Un autre agent reprend `main` dans `epic/50-corrections` en ce moment : **l'étape 0 relit les lignes citées après
  cette reprise**.
- **Ce qui doit rester vrai** : `details.confirmable` et les messages aux mêmes valeurs dans les quatre cases ; l'audit
  `supplier_invoice.duplicate_confirmed` dans les mêmes cas ; la mécanique du bandeau (qui l'allume, qui l'éteint, le rejeu des seuls
  `GET`) ; la table d'états E1 à E9 de la 48-12b (inchangée).
- **Hors périmètre** : #1404 (lot A, sorti vers la v0.21.0, décision du Project Lead ; l'étude du lot A — route
  `GET /api/v1/imported-supplier-invoices/{id}`, relecture et montage de `new/+page.svelte`, inventaire, et l'angle mort de la
  pagination en mémoire de l'outil MCP `kesh_pieces`, portée depuis le 2026-11-24 par **#1509** — se lit dans le premier commit de
  cette fiche, `4978f345c`, et dans `kesh-gate-logs/e50solde/inv-1404.txt`) ; #1415, #1416, #1453, #1491 (P4, sous le plafond) ; #1441,
  #1442, #1443 (décision de Guy du 2026-11-20, Q2 de la 48-11, § *Périmètre*).
- **Références** : fiches `48-12a-doublons-fournisseurs-serveur.md` (D1, D5, D6bis, D7), `48-12b-doublons-fournisseurs-ecran.md`
  (table d'états, E6) ; `docs/testing.md` ; `docs/agents-orchestration.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

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
