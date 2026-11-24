# Story 50-solde : Solde de la dette de l'Epic 50 — relecture d'un document par identifiant (#1404), une seule règle de levée d'un refus de doublon (#1492), un bandeau « base indisponible » qui ne promet pas de réessai (#1414)

Status: drafted — fiche écrite le 2026-11-24 ; **validation P1 à lancer** (prompt `50-solde-dette-epic-validation-prompt-p1.md`) ; passe à `ready-for-dev` **après** la validation seulement (consigne de l'orchestrateur). Signal de splitting déclaré (§ *Règle de splitting*) : arbitrage du Project Lead attendu avant le développement.

*Fiche rédigée le 2026-11-24 (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-50-solde`, branche `story/50-solde-dette-epic`, partie de
`origin/epic/50-corrections` à **`48a66ca24`**. **Tout le code cité est lu à `48a66ca24`.** Release visée : **0.20.0**, vendredi 2026-11-27
(coupure le jeudi 2026-11-26 au soir). **Aucune migration.** Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e50solde/` (inventaires
`inv-1404.txt`, `inv-1492.txt`, `inv-1414.txt`, régénérables par les commandes qu'ils portent en tête). Clé courte des bases : `e50s`
(`kesh_e50s`, `kesh_ci_e50s`, `kesh_e2e_e50s`).*

## Story

En tant que comptable qui complète ses documents à traiter et travaille parfois sur une base qui flanche,
je veux que la relecture d'un document après une réponse perdue ne charge plus tout l'historique des documents complétés, que la
règle « ce refus de doublon se lève-t-il par une confirmation ? » n'ait qu'une source, et que le bandeau « base indisponible » ne me
promette pas un réessai qui n'aura pas lieu pour mon enregistrement,
afin que l'écran reste rapide quand l'historique grossit, que le serveur ne puisse pas dire « confirmable » d'un refus qu'il maintient,
et que je vérifie mon opération au lieu de croire qu'elle sera rejouée.

## Périmètre : la dette des revues de l'Epic 50, plafonnée à trois issues

Story de solde (`CLAUDE.md` § *Priorités des défauts* : « seulement la dette des stories de l'epic, plafonnée par un nombre d'issues
fixé au kickoff ») ; le fichier d'épique dit « plafond fixé à la reprise » : **trois issues**, fixé par l'orchestrateur le 2026-11-24,
choisies **par priorité, puis par ancienneté**, parmi la dette ouverte par les revues des stories 48-11, 48-12a et 48-12b.

| Issue | Labels | Origine | Retenue |
|---|---|---|---|
| **#1404** | `technical-debt`, P3 | validation P2 de la 48-12b (V2-9), dette nommée en D2 de la fiche 48-12b | **oui** (lot A) |
| **#1492** | `technical-debt`, P3 | revue de code P3 de la 48-12a (P3-5) | **oui** (lot B) |
| **#1414** | `bug`, P4 | revue P1 de la 48-12b (lentilles A et E), créée le 2026-11-19 | **oui** (lot C) — la plus ancienne des P4 (#1415 et #1416, du même jour, ont un numéro postérieur) |
| #1441, #1442, #1443 | `bug`, P2 / P4 / P3 | arrondis au pair relevés pendant la 48-11 | **non** : hors périmètre de la 48-11 par décision du Project Lead, ce n'est pas de la dette de revue |
| #1415, #1416 | `bug`, P4 | revue de la 48-12a (formes Unicode du numéro) | **non** : P4, sous le plafond |
| #1453 | `documentation`, P4 | revue de la 48-12b (doc-comment de `unknown-outcome.ts`) | **non** : P4, sous le plafond |
| #1491 | `bug`, P4 | revue de la 48-12a (libellé d'audit) | **non** : P4, sous le plafond |

Les trois lots sont **indépendants** : aucun ne lit le code d'un autre ; ils se développent, se testent et se relisent séparément, et
la fiche est écrite pour qu'un lot s'en retire sans toucher aux deux autres (§ *Règle de splitting*).

## Ce qui se lit au code (`48a66ca24`)

### Lot A — #1404 : un document lu par identifiant à travers une liste entière

1. **Aucune lecture unitaire d'un document à traiter n'existe.** Les routes montées (`crates/kesh-api/src/lib.rs:769-790`) :
   `GET /api/v1/imported-supplier-invoices` (`get(routes::imported_supplier_invoices::list_imported)`, `:771`), puis seulement
   `/{id}/complete`, `/{id}/discard`, `/{id}/source-document`, `/{id}/supplier-candidates`. Le commentaire d'en-tête de la page le dit :
   `frontend/src/routes/(app)/supplier-invoices/new/+page.svelte:10-11` — « Le document se lit dans la liste des documents à
   traiter (aucune lecture unitaire n'existe) ».
2. **La liste n'a pas de borne.** `crates/kesh-db/src/repositories/imported_supplier_invoices.rs:589-603` (`list_by_status`) :
   `"SELECT * FROM imported_supplier_invoices WHERE company_id = ? AND status = ? ORDER BY created_at DESC, id DESC"`, `fetch_all`,
   sans `LIMIT`. Servie telle quelle par `services::pieces::lister_tout` (`crates/kesh-api/src/services/pieces.rs:153-169`), que la route
   appelle (`routes/imported_supplier_invoices.rs:90-103`).
3. **La relecture après refus** (`new/+page.svelte:166-182`, `relireApresRefus`, Story 48-12b D2) :
   ```ts
   const toComplete = await listImported('to_complete', { retry: false });   // :171
   if (toComplete.some((d) => d.id === current.id)) return { kind: 'pending' };
   const completed = await listImported('completed', { retry: false });       // :173
   const found = completed.find((d) => d.id === current.id);
   if (found === undefined) return { kind: 'gone' };                         // :175
   if (typeof found.supplierInvoiceId !== 'number') return { kind: 'unknown' }; // :179
   locked = true;
   return { kind: 'completed', invoiceId: found.supplierInvoiceId };         // :181
   ```
   La seconde lecture rend **tous** les documents complétés de la société — le coût croît avec l'historique (#1404). Elle n'a lieu que
   pour `IMPORT_NOT_PENDING_COMPLETION` et une issue inconnue (`:169-170`) ; une lecture en échec lève et le formulaire passe en E6
   (doc-comment `:158-165`).
4. **Le montage de la page a le même symptôme** (hors du texte de l'issue, dans la classe) : `new/+page.svelte:117`
   `documentId === null ? Promise.resolve(null) : listImported('to_complete'),` puis `:126`
   `const found = toComplete.find((r) => r.id === documentId) ?? null;` — toute la liste `to_complete` pour un document. Un paramètre
   illisible vaut `-1` (`:66-67`) et donne « plus à traiter » ; inconnu, autre société, traité ou écarté donnent **le même** message
   (`documentGone`, `:128` ; voulu, `:11-13`).
5. **Le patron d'une lecture par identifiant existe déjà côté serveur** : `get_supplier_candidates`
   (`routes/imported_supplier_invoices.rs:406-415`) fait `get_company_for` puis
   `imported_supplier_invoices::find_by_id_scoped(&state.pool, company.id, id).await?.ok_or(AppError::ImportedInvoiceNotFound)?`
   (`:412-414`) ; `find_by_id_scoped` (`imported_supplier_invoices.rs:119-131`) exécute `FIND_BY_ID_SQL` (`:29-30`,
   `"SELECT * FROM imported_supplier_invoices WHERE id = ? AND company_id = ?"`) — **la même projection `SELECT *`** que la liste, donc
   la même forme sérialisée (`ImportedSupplierInvoice`, `#[serde(rename_all = "camelCase")]`,
   `crates/kesh-db/src/entities/imported_supplier_invoice.rs:29-30`). `AppError::ImportedInvoiceNotFound` rend
   `404 IMPORTED_INVOICE_NOT_FOUND` (`crates/kesh-api/src/errors.rs:4346-4349`).
6. **RBAC et clés** : les routes du dossier d'import sont montées dans `comptable_routes` (`lib.rs:579`, bloc `:765-790`) ; une clé
   `read` passe pour une méthode sûre (`crates/kesh-api/src/middleware/auth.rs:145` :
   `if scope == kesh_db::entities::ApiKeyScope::Read && !is_safe_method(req.method())`). Le registre d'audit
   (`crates/kesh-api/tests/audit_route_registry.rs`) n'inscrit que les routes **mutantes** : un `GET` n'y entre pas (à confirmer à
   l'étape 0 en lançant le binaire après l'ajout).
7. **Les tests qui fixent la forme actuelle** : `new-page.test.ts:279-296` (« `to_complete` PUIS `completed`, sans rejeu », assertion
   `listImportedMock.mock.calls.slice(1)` égale à `[['to_complete', { retry: false }], ['completed', { retry: false }]]`), et les
   autres cas de relecture `:300-410` (mocks `listImportedMock.mockResolvedValueOnce([])…`) ; le montage, `:120-197`. L'E2E
   `frontend/tests/e2e/supplier-invoice-duplicate.spec.ts:529` intercepte `'**/api/v1/imported-supplier-invoices?status=*'` pour faire
   échouer la relecture (E6 à la complétion) et le retire `:540`. Les E2E E9 (`:237-273`) et E7 (`:275-302`) jouent la relecture contre
   le vrai serveur.

**Inventaire par le symptôme** (« un document lu par son identifiant à travers une liste », `inv-1404.txt`) :

```
$ git grep -nE "listImported\(" -- frontend/src frontend/tests
AttachDocumentPicker.svelte:61:   rows = await listImported('to_complete');
imported-supplier-invoices.api.ts:53:   export async function listImported(
supplier-invoices/import/+page.svelte:187:   toComplete = await listImported('to_complete');
supplier-invoices/new/+page.svelte:117:   documentId === null ? Promise.resolve(null) : listImported('to_complete'),
supplier-invoices/new/+page.svelte:171:   const toComplete = await listImported('to_complete', { retry: false });
supplier-invoices/new/+page.svelte:173:   const completed = await listImported('completed', { retry: false });
(+ deux appels du test de l'API, :27 et :35)
$ git grep -nE "list_by_status|lister_tout\(" -- crates   → routes/imported_supplier_invoices.rs:95, services/pieces.rs:168, :181 (+ tests)
```

| Site | Ce qu'il fait | Classement |
|---|---|---|
| `new/+page.svelte:171-181` (relecture) | cherche **un** document dans `to_complete` puis dans `completed` | **corrigé** (D-A3) |
| `new/+page.svelte:106-143` (montage) | cherche **un** document dans `to_complete` | **corrigé** (D-A4) |
| `AttachDocumentPicker.svelte:61` | affiche la liste `to_complete` pour qu'on y **choisisse** un document | hors classe : une liste montrée |
| `supplier-invoices/import/+page.svelte:187` | affiche la liste « Documents à traiter » | hors classe : une liste montrée |
| `services/pieces.rs:181` (`lister`, outil MCP `kesh_pieces`) | lit toute la liste puis la **tranche en mémoire** (`skip`/`take`, `:186-190`) | hors classe (une page, non un document) ; même coût linéaire : **angle mort assumé**, issue à ouvrir par l'orchestrateur s'il la juge utile (aucune ouverte : recherche `gh issue list` sur « pièces|kesh_pieces|pagin », 2026-11-24) |

### Lot B — #1492 : la levée d'un refus de doublon jugée deux fois

1. **`kesh-db`** — `crates/kesh-db/src/repositories/supplier_invoices.rs:658-664`, dans `guard_duplicates_in_tx` :
   ```rust
   let liftable = match (finding.rule, gesture) {
       (DuplicateRule::SameNumberSameYear, DuplicateGesture::Entry) => false,
       (DuplicateRule::SameDateSameTotal, _) | (_, DuplicateGesture::Unvalidation) => true,
   };
   if liftable && policy == DuplicatePolicy::Confirmed {
   ```
   puis le refus, `:665-673` (`SameNumberSameYear` → `DbError::SupplierInvoiceDuplicate { existing, gesture }`, `SameDateSameTotal` →
   `DbError::SupplierInvoiceProbableDuplicate { existing, gesture }`). Trois appelants : `:944`, `:1152`, `:1875` (ce dernier avec
   `DuplicateGesture::Unvalidation`, `:1883`).
2. **`kesh-api`** — `crates/kesh-api/src/errors.rs:6253-6260`, `supplier_invoice_duplicate_response(code, existing, gesture, probable)` :
   `let confirmable = probable || gesture == DuplicateGesture::Unvalidation;` (`:6260`), écrit dans `details.confirmable` (`:6283`) ;
   appelé par les deux bras `:5583-5598` avec `probable` à `false` puis `true`.
3. Les deux tables concordent aujourd'hui (quatre cases : (règle 1, saisie) → non ; (règle 2, saisie) → oui ; (règle 1 ou 2,
   dévalidation) → oui) ; rien ne les lie. **L'écran ne décide rien** : il lit `details.confirmable`
   (`frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts:592-603`, `:619`) — il n'est pas un troisième site.
4. **Les tests qui tiennent les cases aujourd'hui** : `crates/kesh-db/tests/supplier_invoice_duplicate.rs` (refus levé ou non, par
   exemple `:1136` pour (règle 2, dévalidation)) ; `crates/kesh-api/tests/supplier_invoice_duplicate_e2e.rs:238` (`"confirmable": false`,
   règle 1 à la saisie), `:263` (règle 2 à la saisie, `true`), `:421` (règle 1 à la dévalidation, `true`). **La case (règle 2,
   dévalidation) côté `details.confirmable` : à confirmer à l'étape 0** (aucune assertion trouvée dans `supplier_invoice_duplicate_e2e.rs`
   par `grep -n confirmable`). Un module de tests unitaires existe dans `kesh-api/src/errors.rs` (`:6587`), avec l'aide
   `response_body(resp: Response) -> (StatusCode, serde_json::Value)` (`:6605`) ; un autre dans `supplier_invoices.rs` (`:3838`).

**Inventaire par le symptôme** (« une décision sur la levée d'un refus de doublon », `inv-1492.txt`) :
`git grep -nE "liftable|confirmable|DuplicateGesture::Unvalidation|probable \|\|" -- 'crates/*/src/*'` et
`git grep -nE "confirmable" -- frontend/src ':!*.test.ts'`. **Sites qui décident** : deux, `supplier_invoices.rs:658-661` et
`errors.rs:6260` — tous deux corrigés (D-B1, D-B2). **Sites qui lisent ou décrivent** : `supplier_invoices.rs:1883` (passe le geste,
ne décide pas) ; doc-comments qui récitent la table — `kesh-db/src/errors.rs:574`, `:1892-1894`, `:1905`,
`supplier_invoices.rs:449-460`, `:1567`, `kesh-api/src/errors.rs:5578-5581`, `:6247-6251` — réécrits pour **nommer la source** (D-B3) ;
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
   (réponses NETWORK_ERROR / TIMEOUT / 503 sur les requêtes idempotentes traversant `api-client.ts`) » : faux depuis que le `catch` et le
   503 allument le bandeau pour toute méthode (point 2).
6. **Les tests** : aucun test unitaire de `DegradedBanner.svelte` (`git grep -ln "degraded-banner" frontend/src` ne rend que le
   composant) ; l'E2E `frontend/tests/e2e/db-resilience.spec.ts:59-61` vérifie
   `toContainText(/Base de données temporairement indisponible/)` — le **début** du texte, qui ne change pas. La garde
   `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts` rougit si le repli Svelte diverge de la valeur `fr-CH` d'une clé présente
   au catalogue : elle tient l'alignement du repli et du catalogue.

**Inventaire par le symptôme** (« un texte qui promet un réessai », `inv-1414.txt`) :
`git grep -niE "réessai automatique|retrying automatically|Wiederholungsversuch|tentativo automatico" -- crates frontend docs/manual`
→ les quatre catalogues `:88`, `DegradedBanner.svelte:16` et un commentaire d'E2E (`supplier-invoice-unknown-outcome.spec.ts:161`,
« bandeau « réessai automatique » ») ; `grep -niE "réessai|réessaie|rejou|reconnex" crates/kesh-i18n/locales/fr-CH/messages.ftl` → la
seule ligne 88. **Ensemble clos** : la clé et son repli (corrigés, D-C1), le commentaire d'E2E (réécrit, D-C1). Hors code produit :
`CHANGELOG.md:1711` et `_bmad-output/planning-artifacts/epic-10.md:207` citent l'ancien texte — **historique, inchangé**.

### Manuels

- **Lot A** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` cite `imported-supplier-invoices`,
  `…/{id}/complete` et `…/{id}/supplier-candidates` ; le paragraphe « Pour les intégrations » de la 0.20.0
  (`docs/manual/fr/admin-manual.tex:2585`, section `sec:maj-0-20-0`, `:2575`) recense les changements de contrat de la release : la route
  neuve s'y ajoute (D-A5). Le guide de l'API, `docs/api-external.md` § *Documents à traiter* (`:999`, liste `:1011`), la décrit (D-A5).
  Le manuel utilisateur ne décrit pas la relecture (`grep -n -i "relit\|relecture" docs/manual/fr/user-manual.tex` : rien sur les
  documents à traiter) ; les états E6 à E9 qu'il décrit (`user-manual.tex:2715-2717`, Story 48-12b) **ne changent pas**.
- **Lot B** : rien de visible ; l'admin-manual `:2585` décrit `confirmable` (« jamais le premier à la création, au \texttt{PUT} et à la
  complétion …, toujours le second, et les deux au repassage d'une annulée ») : **reste vrai**, aucune réécriture.
- **Lot C** : aucun manuel ne cite le bandeau (`grep` des quatre formulations sur les `.tex` et sur les PDF aplatis : vide ;
  `ls docs/manual/*/` : seul `fr` porte des `.tex`). Rien à écrire.

## Décisions

### Lot A — #1404

- **D-A1 — une route de lecture unitaire** : `GET /api/v1/imported-supplier-invoices/{id}`, handler `get_imported` dans
  `crates/kesh-api/src/routes/imported_supplier_invoices.rs`, au patron de `get_supplier_candidates` (`get_company_for`,
  `find_by_id_scoped`, `ok_or(AppError::ImportedInvoiceNotFound)`), montée dans `comptable_routes` à côté de la liste (`lib.rs:769-772`).
  Rend `200` et le document **quel que soit son statut** (`to_complete`, `completed`, `discarded`), dans la forme exacte d'un élément de
  la liste ; `404 IMPORTED_INVOICE_NOT_FOUND` pour un document inconnu **ou d'une autre société** (anti-IDOR, même réponse) ; rôle
  Comptable et Administrateur, clés `read` et `read-write` (méthode sûre). Aucune trace d'audit (lecture). Doc-comment du module
  (`:1-15`) complété. *Ajout au contrat, sans rupture* (`CLAUDE.md` § *Versions de l'API* : « un ajout ne change rien pour le client »).
  **Options écartées** : un filtre `?id=` sur la liste (mélange deux contrats sur une route dont `status` est obligatoire, et rend une
  liste pour un objet) ; un passage par le service `pieces` (`ContexteAcces`) — il sert à partager la règle d'accès avec l'outil MCP,
  qui n'a pas besoin de cette lecture ; l'ajouter là quand un outil MCP la demandera.
- **D-A2 — le client** : `getImported(id: number, call?: CallOptions): Promise<ImportedSupplierInvoice>` dans
  `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.api.ts`, `apiClient.get(`/api/v1/imported-supplier-invoices/${id}`, call)`,
  doc-comment au patron de `listImported` (`:50-62`). `listImported` reste (deux autres appelants, tableau du lot A).
- **D-A3 — la relecture lit le document, une fois** : `relireApresRefus` (`new/+page.svelte:166-182`) remplace ses deux listes par
  `getImported(current.id, { retry: false })`, puis :

  | Lecture | Rendu | État du formulaire (48-12b) |
  |---|---|---|
  | `404` `IMPORTED_INVOICE_NOT_FOUND` | `{ kind: 'gone' }` | E7, message actuel (inchangé : `new-page.test.ts:324`) |
  | `status === 'to_complete'` | `{ kind: 'pending' }` | E7 « pour l’instant » (inchangé : `:315`) |
  | `status === 'completed'` et `typeof supplierInvoiceId === 'number'` | `locked = true` ; `{ kind: 'completed', invoiceId }` | E9 |
  | `status === 'completed'`, `supplierInvoiceId` non numérique | `{ kind: 'unknown' }` | E6 |
  | `status === 'discarded'` | `{ kind: 'gone' }` | E7, message actuel (inchangé) |
  | tout autre `status` (la contrainte `chk_imported_si_status` l'interdit) | `{ kind: 'unknown' }` | E6 |
  | toute autre erreur (5xx, coupure, délai, `ApiError` d'un autre code) | **lève** | E6 (inchangé) |

  Le `404` se reconnaît par `isApiError(err) && err.code === 'IMPORTED_INVOICE_NOT_FOUND'` ; **seul ce code** vaut « gone » : un `404`
  d'un autre code (route absente derrière un proxy, par exemple) lève et donne E6, l'état prudent. Ce tableau rend exactement ce que
  rendait la double lecture pour chaque état du document (« discarded » n'était ni dans `to_complete` ni dans `completed` → `gone`) :
  **aucun état visible ne change**. La question d'ordre de la V2-9 de la 48-12b (un document qui passe de `to_complete` à `completed`
  entre deux lectures) **disparaît** : une seule lecture, d'une seule ligne. L'angle mort du détachement (48-12b, D2) reste tel quel.
  Doc-comment `:158-165` réécrit. **Option écartée** : garder `to_complete` en liste et lire `completed` par identifiant (deux requêtes
  pour un objet, et la liste `to_complete` reste sans borne).
- **D-A4 — le montage lit le document** : `onMount` (`new/+page.svelte:106-143`) remplace `listImported('to_complete')` par
  `getImported(documentId)` (rejeu par défaut des `GET`, comme aujourd'hui) **quand `documentId` est un entier positif** ; `documentId`
  `-1` (paramètre illisible, `:66-67`) donne `documentGone` **sans requête**. `status !== 'to_complete'` → `documentGone` ;
  `404 IMPORTED_INVOICE_NOT_FOUND` → `documentGone` (même message pour inconnu, autre société, traité ou écarté, comme `:11-13`) ; toute
  autre erreur → `loadError` (inchangé). Le commentaire d'en-tête `:8-13` se réécrit (« le document se lit par son identifiant »).
  **Option écartée** : laisser le montage à la liste (il garde le coût que l'issue dénonce, sur `to_complete`, et le commentaire
  « aucune lecture unitaire n'existe » deviendrait faux).
- **D-A5 — la documentation de la route** : `docs/api-external.md` § *Documents à traiter* — un paragraphe
  **`GET /api/v1/imported-supplier-invoices/{id}`** après celui de la liste (`:1011`), *depuis la v0.20.0* : lecture, même forme qu'un
  élément de la liste, tout statut, `404 IMPORTED_INVOICE_NOT_FOUND` (inconnu ou autre société) ; `docs/manual/fr/admin-manual.tex`,
  paragraphe « Pour les intégrations » de `sec:maj-0-20-0` (`:2585`) — une phrase : « \texttt{GET /api/v1/\allowbreak
  imported-supplier-invoices/\allowbreak\{id\}} est \textbf{nouveau} : il rend un document à traiter, complété ou écarté, par son
  identifiant. » Les PDF ne se commitent pas (`CLAUDE.md` § *Fusions et fichiers communs*).

### Lot B — #1492

- **D-B1 — la table vit dans `kesh-db`, en un seul endroit** : une méthode publique
  `DuplicateRule::lifted_by_confirmation(self, gesture: DuplicateGesture) -> bool` dans
  `crates/kesh-db/src/repositories/supplier_invoices.rs` (à côté de `DuplicateRule::number`, `:494-502`), qui porte le `match` de
  `:658-661` **tel quel** et le doc-comment de la table (quatre cases, D1/D6bis de la 48-12a). `guard_duplicates_in_tx` l'appelle
  (`if finding.rule.lifted_by_confirmation(gesture) && policy == DuplicatePolicy::Confirmed`). Le `match` reste **exhaustif sur le
  couple** (pas de `_ => …` qui avalerait un geste ou une règle ajoutés demain).
- **D-B2 — `kesh-api` relit la table, il ne la réécrit pas** : `supplier_invoice_duplicate_response` reçoit
  `rule: DuplicateRule` à la place de `probable: bool` ; `confirmable = rule.lifted_by_confirmation(gesture)` ; le message se choisit sur
  `(gesture, rule)`. Les deux bras (`errors.rs:5583-5598`) passent `DuplicateRule::SameNumberSameYear` et
  `DuplicateRule::SameDateSameTotal`. La correspondance variante ↔ règle tient en deux `match` exhaustifs (`kesh-db` `:665-673`, règle →
  variante ; `kesh-api`, variante → règle), que le compilateur et les tests de l'AC-B2 (code **et** `confirmable` sur les quatre cases)
  lient. **Options écartées** : un champ `confirmable` dans les deux variantes de `DbError` (redondant avec `gesture` : un état
  incohérent devient constructible, et les tests de `kesh-db` qui comparent des `Debug` de variantes changent tous) ; une variante unique
  `SupplierInvoiceDuplicate { rule, existing, gesture }` (change `error_code`, `errors.rs:2606-2607`, et une vingtaine de motifs de tests :
  hors de la mesure d'une story de solde).
- **D-B3 — les doc-comments nomment la source** : ceux qui récitent la table (`kesh-db/src/errors.rs:574`, `:1892-1894`, `:1905` ;
  `supplier_invoices.rs:449-460`, `:1567` ; `kesh-api/src/errors.rs:5578-5581`, `:6247-6251`) gardent leur résumé et renvoient à
  `DuplicateRule::lifted_by_confirmation` — sans en changer le sens.

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
    un’operazione, verifica se è stata registrata. »
  Chaque phrase est vraie pour une lecture comme pour une écriture (point 4 du lot C : le bandeau s'éteint de lui-même) ; aucune ne dit
  « réessai ». Apostrophe typographique U+2019 dans `fr-CH` et `it-CH`, comme les clés récentes. Le doc-comment
  `api-health.svelte.ts:4-6` se corrige (« sur toute requête traversant `api-client.ts` : rejeu pour les méthodes idempotentes
  seulement ») ; le commentaire d'E2E `supplier-invoice-unknown-outcome.spec.ts:161` cesse de citer « réessai automatique ».
  **Options écartées** : ne plus allumer le bandeau pour une écriture (masquerait une panne réelle, et le bandeau suivant viendrait du
  premier `GET`) ; deux textes selon la méthode (l'état `apiHealth` est **global** et ne sait pas quelle requête l'a allumé : un `GET`
  concurrent le rallume) ; le texte proposé par l'issue (« … — vérifiez l'état de votre dernière opération avant de recommencer ») —
  repris en substance, avec la phrase qui dit au lecteur que le bandeau s'en ira seul (ce que l'ancien « réessai » cherchait à dire).

## Critères d'acceptation

### Lot A — #1404

1. **AC-A1 — la route** (`crates/kesh-api/tests/inbox_import_e2e.rs`, tests neufs) : `GET /api/v1/imported-supplier-invoices/{id}`
   rend `200` et un corps **égal** (`serde_json::Value`) à l'élément de même `id` de `GET …?status=<son statut>`, pour un document
   `to_complete`, un document `completed` (avec `supplierInvoiceId`) et un document `discarded` ; `404` et `error.code ==
   "IMPORTED_INVOICE_NOT_FOUND"` pour un identifiant inconnu **et** pour un document d'une autre société ; `403` pour le rôle
   Consultation ; `200` pour une clé `read` d'un créateur Comptable.
2. **AC-A2 — le client** (`imported-supplier-invoices.api.test.ts`) : `getImported(7, { retry: false })` appelle
   `apiClient.get('/api/v1/imported-supplier-invoices/7', { retry: false })` ; sans options, le second argument est `undefined`.
3. **AC-A3 — la relecture** (`new-page.test.ts`, cas réécrits) : chaque ligne du tableau de D-A3 a son test (sept), qui vérifie l'état
   rendu (E9 et son lien, E6, E7 « pour l’instant », E7 message actuel) **et** `getImportedMock` appelé une fois avec `(docId, { retry: false })` après
   l'échec de la complétion ; **`listImportedMock` n'est jamais appelé** par la relecture. Les assertions de comportement des tests
   existants (`:279-410` : E9 figé, lien nouvel onglet, ancien message absent, E8 figé pendant la lecture, E6 → reprise → E9 V4-3) sont
   conservées, seules les données de mock changent ; l'assertion d'ordre `['to_complete'…], ['completed'…]` est **retirée** (D-A3).
4. **AC-A4 — le montage** (`new-page.test.ts`, cas `:120-197` réécrits) : `?document=7` appelle `getImported(7)` et jamais
   `listImported` ; `document=abc` ne fait **aucune** requête de document et affiche `supplier-invoice-new-doc-gone` ; un `404
   IMPORTED_INVOICE_NOT_FOUND` et un document `completed` ou `discarded` affichent `supplier-invoice-new-doc-gone` (même texte) ; une
   erreur `500` affiche `supplier-invoice-new-error` ; un document sans QR ne déclenche aucune proposition (cas `:178-197` conservé).
5. **AC-A5 — de bout en bout** (`supplier-invoice-duplicate.spec.ts`) : les scénarios E9 (`:237`) et E7 (`:275`) restent verts contre
   le vrai serveur **et** vérifient, par `page.on('request')`, qu'aucune requête `…/imported-supplier-invoices?status=` n'est partie
   entre le clic « Enregistrer » et l'avis ; le volet « E6, complétion » du test de recette visuelle (`:363`, lignes `:517-540`) intercepte désormais
   `` `**/api/v1/imported-supplier-invoices/${docId}` `` (méthode `GET` seulement, `route.fallback()` sinon) et reste vert.
6. **AC-A6 — documentation** : `docs/api-external.md` porte le paragraphe de D-A5 (`git grep -n "imported-supplier-invoices/{id}\*\*"
   docs/api-external.md` le rend) ; `admin-manual.tex`, section `sec:maj-0-20-0`, porte la phrase de D-A5 ; `make fr` dans
   `docs/manual/` compile (PDF non commité) et `pdftotext … | tr '\n' ' ' | tr -s ' '` y trouve « imported-supplier-invoices/{id} est
   nouveau » (au contrôle de la release, la régénération des PDF étant faite à la release).
7. **AC-A7 — l'inventaire tenu** : `git grep -nE "listImported\(" -- frontend/src ':!*.test.ts'` ne rend plus que
   `AttachDocumentPicker.svelte:61`, `import/+page.svelte:187` et la définition ; `git grep -n "aucune lecture unitaire" -- frontend`
   est vide. Sortie copiée au Dev Agent Record.

### Lot B — #1492

8. **AC-B1 — la table, une fois** (`supplier_invoices.rs`, module `tests`) : un test pur sur les **quatre** cases de
   `DuplicateRule::lifted_by_confirmation` — (règle 1, `Entry`) → `false`, (règle 2, `Entry`) → `true`, (règle 1, `Unvalidation`) →
   `true`, (règle 2, `Unvalidation`) → `true`.
9. **AC-B2 — la réponse relit la table** (`kesh-api/src/errors.rs`, module `tests`, avec `response_body`) : pour chacune des quatre
   cases, `AppError::Database(DbError::Supplier…Duplicate { existing, gesture })` rend `409`, le bon `error.code`, et
   `details.confirmable == DuplicateRule::<règle>.lifted_by_confirmation(gesture)` **et** égal à la valeur littérale attendue (les deux :
   l'une prouve le lien, l'autre la valeur). Les tests e2e existants (`supplier_invoice_duplicate_e2e.rs:238`, `:263`, `:421`) restent
   verts.
10. **AC-B3 — plus aucune seconde table** :
    `git grep -nE "let liftable|probable \|\||gesture == DuplicateGesture::Unvalidation" -- 'crates/*/src/*'` est **vide** ;
    `git grep -nw lifted_by_confirmation -- 'crates/*/src/*'` ne rend que la définition, son appel dans `guard_duplicates_in_tx`, son
    appel dans `supplier_invoice_duplicate_response`, les doc-comments de D-B3 et les tests. Sortie copiée.
11. **AC-B4 — rien ne change au comportement** : `crates/kesh-db/tests/supplier_invoice_duplicate.rs`,
    `supplier_invoice_duplicate_lock.rs` et `crates/kesh-api/tests/supplier_invoice_duplicate_e2e.rs` passent **sans modification**
    (`git diff origin/epic/50-corrections...HEAD --stat -- <ces trois fichiers>` vide).

### Lot C — #1414

12. **AC-C1 — le texte** : les quatre catalogues portent, à la clé `db-unavailable-banner`, les textes de D-C1, mot pour mot ; le
    repli de `DegradedBanner.svelte` égale la valeur `fr-CH` ; `i18n-repli-divergent-actif.test.ts` et `npm run lint-i18n-ownership`
    verts.
13. **AC-C2 — le composant** (`frontend/src/lib/shared/components/DegradedBanner.test.ts`, neuf) : après `apiHealth.setDegraded()`,
    `degraded-banner` affiche le texte français de D-C1 ; il ne contient ni « réessai » ni « automatique » ; après
    `apiHealth.clearDegraded()`, il disparaît (test de filet, déjà vrai). Nettoyage de l'état global entre tests
    (`clearDegraded()` en `afterEach`).
14. **AC-C3 — plus aucune promesse** :
    `git grep -niE "réessai automatique|retrying automatically|Wiederholungsversuch läuft|tentativo automatico" -- crates frontend docs/manual`
    est **vide**. Sortie copiée.
15. **AC-C4 — de bout en bout et recette visuelle** : `db-resilience.spec.ts` (scénarios 1 et 2) vert, avec une assertion ajoutée
    `toContainText('vérifiez si elle a été enregistrée')` ; **captures** (règle *Recette visuelle* du `CLAUDE.md`, thème clair, base de
    démonstration) du bandeau seul (scénario 2 de `db-resilience`) et du bandeau **avec** l'avis E6 de la complétion (même montage que la
    capture `e4812b-E6-completion` de la 48-12b), sous `kesh-gate-logs/e50solde/captures/`, jointes au Dev Agent Record et au prompt de
    la revue de code.

### Transverses

16. **AC-T1 — aucune facture, aucun avoir, aucune écriture ne change** : aucun fichier de `crates/kesh-qrbill`, `crates/kesh-report`,
    `crates/kesh-core`, ni `crates/kesh-db/migrations/` n'est touché (`git diff origin/epic/50-corrections...HEAD --stat -- crates/kesh-qrbill
    crates/kesh-report crates/kesh-core crates/kesh-db/migrations` vide) ; les seuls changements de `kesh-db` sont ceux de D-B1/D-B3
    (aucun SQL modifié : `git diff … -- crates/kesh-db | grep -E '^[+-].*(SELECT|UPDATE|INSERT|DELETE)'` vide).
17. **AC-T2 — gates** (§ *Gates*) verts au dernier commit de code ; E2E complet « vert » au sens du `CLAUDE.md` (seuls les échecs
    attendus, jugés fichier par fichier).

## Tests à écrire, et la preuve du rouge

Journaux attendus sous `/home/gcorbaz/devel/kesh-gate-logs/e50solde/dev/`.

| Test | Rouge sans le correctif | Preuve |
|---|---|---|
| AC-A1 (route) | **rouge d'abord** : la route n'existe pas (`404` sans `error.code` attendu, ou `405`) | `rouge-api.log` |
| AC-A2 (client) | rouge d'abord : `getImported` n'existe pas (échec d'import) | `rouge-front.log` |
| AC-A3 (relecture) | rouge d'abord : la page appelle encore `listImported` ; **M-A1** : la relecture remise à `listImported('completed')` après le correctif → le test « jamais `listImported` » rougit (`mut-a1.log`) ; **M-A2** : le `404` traité comme une erreur (lève → E6) → le test « 404 → gone » rougit (`mut-a2.log`) ; **M-A3** : `discarded` → `pending` → rouge (`mut-a3.log`) ; **M-A4** : `locked = true` posé aussi pour `completed` sans facture → le test « completed sans facture → E6 » rougit (`mut-a4.log`) | journaux nommés |
| AC-A4 (montage) | rouge d'abord : `getImported` jamais appelé ; **M-A5** : `document=abc` refait une requête (`getImported(-1)`) → rouge (`mut-a5.log`) | idem |
| AC-A5 (E2E) | rouge d'abord pour l'assertion « aucune requête `?status=` » (la page en émet deux aujourd'hui) ; le scénario E6 rougit tant que l'interception vise l'ancienne URL après le correctif (la relecture réussit, pas d'E6) | `rouge-e2e.log` (le seul fichier `supplier-invoice-duplicate.spec.ts`) |
| AC-A1 anti-IDOR | **M-A6** : la route montée dans le routeur des lectures (tout rôle) au lieu de `comptable_routes` → le test `403` Consultation rougit (`mut-a6.log`) | — |
| AC-B1 | — (la méthode est neuve) | **M-B1** : (règle 1, `Entry`) → `true` dans `lifted_by_confirmation` → AC-B1 **et** AC-B2 **et** `supplier_invoice_duplicate_e2e.rs:238` rouges **dans la même exécution** — la preuve que les deux crates lisent une seule table (`mut-b1.log`) ; **M-B2** : (règle 2, `Unvalidation`) → `false` → AC-B1, AC-B2 et `kesh-db/tests/supplier_invoice_duplicate.rs:1136` rouges (`mut-b2.log`) |
| AC-B2 | ne rougit pas sur le code d'avant (les valeurs sont les mêmes) : **filet**, et c'est sa raison d'être — il rougit sous M-B1/M-B2 | déclaré |
| AC-C1, AC-C2 | rouge d'abord : AC-C2 rougit sur l'ancien texte ; **M-C1** : l'ancien texte remis dans le seul `fr-CH` → `i18n-repli-divergent-actif` et AC-C2 rouges (`mut-c1.log`) ; **M-C2** : l'ancien texte remis dans le seul repli Svelte → `i18n-repli-divergent-actif` rouge (`mut-c2.log`) | `rouge-front.log` |
| AC-C4 | rouge d'abord pour l'assertion ajoutée | `rouge-e2e.log` (`db-resilience.spec.ts`) |
| AC-A7, AC-B3, AC-C3, AC-B4, AC-T1 | — | commandes et sorties copiées |

Après chaque mutation restaurée, **toucher le fichier** avant de relancer (cargo peut garder le binaire muté).

## Tâches

1. **Étape 0** : relire à la tête de la branche les lignes citées (`new/+page.svelte:106-182` ; `imported-supplier-invoices.api.ts:50-62` ;
   `routes/imported_supplier_invoices.rs:90-103`, `:406-415` ; `lib.rs:765-790` ; `supplier_invoices.rs:494-502`, `:622-674` ;
   `kesh-api/src/errors.rs:5583-5598`, `:6253-6290`, `:6587-6620` ; `messages.ftl:88` des quatre locales ; `DegradedBanner.svelte` ;
   `api-health.svelte.ts:1-15`) ; en copier trois au Change Log ; régénérer les trois inventaires (commandes en tête des fichiers
   `inv-*.txt`) et noter tout écart (une reprise de `main` dans `epic/50-corrections` peut avoir ajouté un site). **Confirmer** : la case
   (règle 2, dévalidation) de `details.confirmable` côté API (lot B, point 4) ; que `audit_route_registry` ne demande rien pour un `GET`
   (lot A, point 6).
2. Tests rouges (AC-A1 à AC-A5, AC-C2, AC-C4), journaux `rouge-*.log`.
3. **Lot B** (le plus petit, `kesh-db` d'abord) : `lifted_by_confirmation`, appel dans `guard_duplicates_in_tx`, `kesh-api` (D-B2),
   doc-comments (D-B3), tests AC-B1, AC-B2 ; mutations M-B1, M-B2 ; **gate complet** (exception `kesh-db` du `CLAUDE.md` : un patch qui
   touche un repository).
4. **Lot A, serveur** : handler, route, doc-comment du module ; tests AC-A1 ; M-A6.
5. **Lot A, écran** : `getImported`, relecture (D-A3), montage (D-A4), commentaires ; tests AC-A2 à AC-A4 ; M-A1 à M-A5 ; E2E AC-A5.
6. **Lot A, documentation** : `api-external.md`, `admin-manual.tex` (D-A5), `.tex` seulement.
7. **Lot C** : quatre catalogues, repli, doc-comment `api-health.svelte.ts`, commentaire d'E2E ; test AC-C2 ; E2E AC-C4 ; M-C1, M-C2 ;
   captures.
8. Propagation (grep du symptôme de chaque lot, AC-A7, AC-B3, AC-C3), AC-T1 ; Dev Agent Record. **Ne pas écrire `CHANGELOG.md` ni
   `sprint-status.yaml`** (l'orchestrateur tient les statuts).

Un commit par étape franchie (test rouge, correctif vert, remédiation), poussé sur `story/50-solde-dette-epic` seule (`CLAUDE.md`
§ *Worktrees et agents*), chacun avec `refs #1404`, `refs #1492` ou `refs #1414` selon le lot ; **aucun mot-clé de fermeture**.

## Gates

- **En cours de développement** (gate ciblé) : `cargo fmt --all -- --check` ;
  `cargo clippy --workspace --all-targets -- -D warnings` ;
  `cargo nextest run -E 'binary(inbox_import_e2e) | binary(supplier_invoice_duplicate_e2e) | binary(supplier_invoice_duplicate) | binary(supplier_invoice_duplicate_lock) | (package(kesh-api) & kind(lib)) | (package(kesh-db) & kind(lib))'`
  sur `kesh_e50s` ; frontend : `npm run check`, `npm run lint-i18n-ownership`, `npx vitest run` sur les fichiers touchés.
  **Exception `kesh-db`** : le lot B touche un repository → **gate complet** au commit qui le porte, même en cours de boucle.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh` (base `kesh_ci_e50s`) ; frontend complet
  (`npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`).
- **À la clôture**, au dernier commit de code : `gate-ci` et **E2E complet** (`docs/testing.md` § *Prérequis Playwright local*,
  `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés, base `kesh_e2e_e50s` ; « vert » = les seuls échecs attendus, jugés
  fichier par fichier).
- Une commande lourde à la fois sous `scripts/mem-guard.sh`, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant, verrou
  `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`. Le story file n'affirme que ce qui a tourné.

## Note de version

**Ajouté**

- API : `GET /api/v1/imported-supplier-invoices/{id}` rend un document à traiter, complété ou écarté, par son identifiant (même forme
  qu'un élément de la liste ; `404 IMPORTED_INVOICE_NOT_FOUND` pour un document inconnu ou d'une autre société) (refs #1404).

**Modifié**

- Le bandeau « Base de données temporairement indisponible » ne promet plus un « réessai automatique » : il dit qu'il disparaîtra quand
  la base répondra, et invite à vérifier si la dernière opération a été enregistrée avant de la recommencer — une saisie enregistrée
  n'est jamais renvoyée d'elle-même (refs #1414).

**Interne**

- La complétion d'un document relit ce seul document après une réponse perdue, au lieu de toute la liste des documents complétés
  (refs #1404).
- La règle qui dit si une confirmation lève un refus de doublon de facture fournisseur n'est plus écrite qu'une fois (refs #1492).

## Règle de splitting — signal au Project Lead

**Modules touchés** (grain de la 48-12b : une crate backend, un domaine ou une route du frontend, ou la documentation ; les tests
suivent leur module) :

| # | Module | Lot | Fichiers de production |
|---|---|---|---|
| 1 | `kesh-db` | B | `repositories/supplier_invoices.rs`, `errors.rs` (doc-comments) |
| 2 | `kesh-api` | A, B | `routes/imported_supplier_invoices.rs`, `lib.rs`, `errors.rs` |
| 3 | `kesh-i18n` | C | `locales/{fr,de,en,it}-CH/messages.ftl` |
| 4 | `features/imported-supplier-invoices` | A | `imported-supplier-invoices.api.ts` |
| 5 | route `supplier-invoices` | A | `routes/(app)/supplier-invoices/new/+page.svelte` |
| 6 | `shared` | C | `components/DegradedBanner.svelte`, `utils/api-health.svelte.ts` (doc-comment) |
| 7 | documentation | A | `docs/api-external.md`, `docs/manual/fr/admin-manual.tex` |

**Sept modules : au-dessus de la borne de cinq.** La règle (`CLAUDE.md` § *Règle de splitting préventif*) demande le découpage ; la
dérogation n'y est ouverte que si le découpage crée des cycles ou des fusions intermédiaires intestables — **ce n'est pas le cas** : les
trois lots sont indépendants. Par lot : A **quatre** modules (2, 4, 5, 7), B **deux** (1, 2), C **deux** (3, 6). Le signal est déclaré ;
**l'arbitrage revient au Project Lead**. Découpages possibles, tous mécaniques (chaque lot a ses D, AC, tests et tâches) :

1. **Deux fiches** : `50-solde-dette-epic` garde les lots A et B (**cinq** modules : 1, 2, 4, 5, 7) ; le lot C devient une fiche à part
   (deux modules), ou passe à la story de correction d'une release suivante. L'Epic 50 compte alors cinq stories (48-11, 48-12a, 48-12b,
   deux de solde), sous la borne de six.
2. **Trois fiches**, une par issue : l'Epic 50 arrive à **six** stories, la borne ; trois boucles de validation et de revue au lieu
   d'une avant la coupure de jeudi soir.
3. **Garder une fiche** par décision expresse du Project Lead (dérogation hors du critère écrit), à écrire alors dans une section
   « Dérogation règle de splitting ».

## Dev Notes

- **Charge estimée** : lot B, une à deux heures ; lot C, une heure et demie avec les captures ; lot A, une demi-journée (route, tests
  serveur, client, page, réécriture des tests de relecture, E2E, deux documents). Une journée de développement, boucles de validation
  et de revue non comprises. Tenable avant la coupure du jeudi 2026-11-26 au soir **si la validation converge en deux ou trois passes** ;
  le lot A est le seul qui puisse la menacer (sa remédiation touche quatre modules, ce qui interdit les passes ciblées). Si le temps
  manque, retirer un lot entier plutôt que le livrer à moitié : C d'abord, puis A (« une issue, une release »).
- **Fichiers communs** : avec les 48-12a/48-12b (déjà closes et fusionnées dans `epic/50-corrections`) — `supplier_invoices.rs`,
  `kesh-api/src/errors.rs`, `new/+page.svelte`, `new-page.test.ts`, `supplier-invoice-duplicate.spec.ts`, `admin-manual.tex`
  (`sec:maj-0-20-0`, où la 49-1a ouvre la même section sur sa branche : à l'intégration, tous les paragraphes sous un seul titre) ;
  `lib.rs` (bloc des routes, commun à tous les epics) ; catalogues i18n (chaque epic dans son bloc ; ici une valeur existante change,
  aucune clé neuve). Un autre agent reprend `main` dans `epic/50-corrections` en ce moment : **l'étape 0 relit les lignes citées après
  cette reprise**.
- **Ce qui doit rester vrai** : la table d'états de la 48-12b (E1 à E9) — aucun état ne change, seule la source de la relecture ; le
  même message pour un document inconnu, d'une autre société, traité ou écarté ; `details.confirmable` aux mêmes valeurs ; la mécanique
  du bandeau (qui l'allume, qui l'éteint, le rejeu des seuls `GET`).
- **Hors périmètre** : la pagination en mémoire de l'outil MCP `kesh_pieces` (angle mort du lot A) ; #1415, #1416, #1453, #1491
  (P4, sous le plafond) ; #1441, #1442, #1443 (décision du Project Lead) ; l'angle mort du détachement (48-12b, D2).
- **Références** : fiches `48-12a-doublons-fournisseurs-serveur.md` (D1, D5, D6bis, D7), `48-12b-doublons-fournisseurs-ecran.md` (D2,
  table d'états, § *Ce qui se lit au code* point 1) ; `docs/testing.md` ; `docs/agents-orchestration.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-11-24 — fiche créée** (Opus 5.5, `bmad-create-story`, en autonomie), à `48a66ca24` de `origin/epic/50-corrections`. Périmètre :
  trois issues fixées par l'orchestrateur (#1404, #1492, #1414), écartées et pourquoi au § *Périmètre*. Inventaires par le symptôme :
  `kesh-gate-logs/e50solde/inv-1404.txt` (51 lignes), `inv-1492.txt` (47), `inv-1414.txt` (33). Site ajouté au texte de #1404 : le
  montage de la page (`new/+page.svelte:106-143`, même symptôme). Prémisses **à confirmer à l'étape 0** : la case (règle 2,
  dévalidation) de `details.confirmable` aux tests de l'API ; l'absence d'exigence d'`audit_route_registry` pour un `GET`. **Signal de
  splitting** : sept modules (§ *Règle de splitting*), arbitrage du Project Lead demandé. Statut `drafted` ; `ready-for-dev` après la
  validation P1 (prompt `50-solde-dette-epic-validation-prompt-p1.md`, Sonnet).
