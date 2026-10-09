# Story 15.1c-0 : Un groupe de lettrage se lit avec ses pièces, et dit d'avance s'il se délettre

## Status

ready-for-dev **après la livraison de la 15-1b** *(et donc de la 15-1b-0, des 15-1a2-0, -i, -ii)* — créée le
2026-10-09 par extraction de la partie serveur de la 15-1c-i, à la remédiation de la validation P2 de la 15-1c
(registre **C-15-1c-14**) ; sa **première** passe de validation a été tenue en **P3 de l'ensemble 15-1c** (avec la
15-1c-i et la 15-1c-ii : 0 au-dessus de LOW, LOW appliqués), puis la P4 ciblée (Haiku : 0 au-dessus de
LOW, LOW appliqués) — **VALIDATION CLOSE**.

⛔ **Ordre** : **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2-0 → 15-1a2-i → 15-1a2-ii → 15-1b-0 → 15-1b →
15-1c-0 → 15-1c-i → 15-1c-ii**. Sur la base `f9b6b199`, seules la 15-1a-i et la 15-1a-ii sont livrées : **rien de
cette story ne se développe avant la 15-1b mergée** — `document_owners` et `DocumentKind` (15-1b-0),
`open_period_rule` (15-1a2-0), la requête B de la vue (15-1b) et le corps de la dissolution déplacé dans
`dissolve_group_inner` (15-1a2-i) n'existent pas avant elles. Le test 2 suppose la 15-1a2-i (groupes `document`).
Le développement se fait sur une branche **rebasée sur `main` après le merge de la 15-1b**.

## Story

**As a** indépendant, PME ou fiduciaire — et l'écran des postes ouverts (15-1c-i), comme toute intégration par clé
d'API,
**I want** que la lecture d'un groupe de lettrage nomme, pour chaque ligne, son journal, son libellé, sa pièce et sa
période, et dise d'avance si le délettrage sera refusé et pourquoi,
**so that** un groupe puisse s'afficher et se défaire sans offrir un bouton voué au refus, et sans qu'un écran
recopie une règle du serveur.

Story **serveur + documentation**, extraite de la 15-1c-i (#518, **`refs #518`** — c'est la **15-1c-ii** qui ferme
l'issue). Aucune route neuve, aucune migration, aucun code d'erreur neuf, aucune clé i18n. Elle enrichit la réponse
de `GET /api/v1/letterings/{key}` et fait passer les refus 1 à 3 de la dissolution manuelle par **une** fonction
d'ordre, que la lecture appelle aussi.

## Contrats consommés (lecture seule)

| contrat | fiche validée | ce que cette story en emploie |
|---|---|---|
| `GET /api/v1/letterings/{key}`, `DELETE /api/v1/letterings/{key}`, `POST /api/v1/letterings` | 15-1a-i (livrée, `routes/letterings.rs`) | le `GET` est **enrichi** ; le `POST` et le `DELETE` gardent leur forme |
| `dissolve_group_in_tx` (corps dans `dissolve_group_inner` après la 15-1a2-i), `first_document_owner`, `find_group`, `group_account_number` | 15-1a-i, 15-1a2-i P3, 15-1b-0 D3 | la dissolution appelle la fonction d'ordre (AC15) ; ses détails de refus sont inchangés |
| `journal_entries::document_owners`, `DocumentKind::{blocks_manual_lettering, reversal_blocker, as_str}` | 15-1b-0 D1 | `document`, `ownedByDocument` |
| `open_period_rule` / `OpenPeriodRule::line_in_open_period` | 15-1a2-0 D1 | `inOpenPeriod` |
| la requête **B** de la vue (`document`, `inOpenPeriod` d'une page) | 15-1b T1, AC3 | sa part « pièce et période » est **factorisée et réemployée** (AC15) |

## Critères d'acceptation

*Numérotation de la 15-1c conservée : AC15 et AC16 viennent de la 15-1c-i, qui ne les porte plus ; AC18 est neuf.*

**AC15 — `GET /api/v1/letterings/{key}` porte de quoi afficher et défaire un groupe** (R-2 = F-2 de la validation
P1 de la 15-1c ; C-15-1c-2, révisée par C-15-1c-15). La réponse livrée (`routes/letterings.rs`, `LetteringResponse`,
partagée par `POST` et `GET` : `key`, `code`, `origin`, `accountId`, et par ligne `id`, `entryId`, `entryNumber`,
`fiscalYearId`, `fiscalYearName`, `date`, `debit`, `credit`) ne permet ni d'écrire « lettrage de la facture F-… », ni
de savoir si « Délettrer » aboutira. Elle gagne, **pour le seul `GET`**, par un DTO distinct
(`LetteringDetailResponse`) :

```json
{
  "key": 27, "code": "AA", "origin": "reversal", "accountId": 7,
  "accountNumber": "2000", "accountName": "…",
  "manualDissolutionBlockedBy": null | "LETTERING_IS_DOCUMENT" | "LETTERING_LINE_OWNED_BY_DOCUMENT" | "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
  "lines": [ {
    "id": 27, "entryId": 3, "entryNumber": 12, "fiscalYearId": 1, "fiscalYearName": "2026",
    "date": "2026-03-01", "debit": "100.0000", "credit": "0.0000",
    "journal": "Achats", "description": "…",
    "document": null | { "type": "supplierInvoice", "id": 9, "number": "FF-12", "invoiceId": null, "invoiceNumber": null },
    "ownedByDocument": true | false,
    "inOpenPeriod": true | false
  } ]
}
```

- **`document` et `inOpenPeriod` : même source que les items de la vue** (validation P3, R L-1). Le partage porte
  sur **deux** pièces, une par crate :
  - **`kesh-db`** — une fonction de `repositories/letterings.rs`, par exemple
    `lines_documents_and_periods(conn, company_id, &[(line_id, entry_id, fiscal_year_id, entry_date)]) ->
    (BTreeMap<entry_id, Vec<DocumentOwner>>, BTreeMap<line_id, bool>)` : **un** appel `document_owners` sur les
    écritures distinctes, `open_period_rule` chargé **une fois** pour les exercices distincts, puis
    `line_in_open_period` par ligne — appelée par `open_items` (part « pièce et période » de la requête B) **et**
    par la lecture d'un groupe. Le **reste dû** (`documentState`, `amountDue`, trois agrégations, 15-1b AC8) n'en
    fait pas partie : la lecture d'un groupe ne le calcule pas.
  - **`kesh-api`** — le constructeur du sous-objet `document` à partir d'un `DocumentOwner` (sérialisé par
    `DocumentKind::as_str()`, 15-1b AC3), écrit **une** fois et appelé par le DTO de la vue et par celui du
    groupe. ⛔ La table de sérialisation reste dans `DocumentKind` ; `kesh-db` ne construit pas de JSON.
  Si la 15-1b n'a pas exposé ces deux pièces ainsi, cette story les extrait (refactor sans changement de
  comportement, prouvé par les tests de la 15-1b **inchangés et verts**). Les noms réels s'écrivent au Dev Agent
  Record. Le test 6 prouve l'**égalité des sorties** ; l'**unicité du code** se prouve par
  `grep -rn "document_owners\|open_period_rule" crates/kesh-db/src crates/kesh-api/src` au Dev Agent Record (un site
  d'appel de chaque pour la vue et le groupe : la fonction partagée).
- **`journal` et `description` ne viennent pas de la requête B** : ce sont des colonnes des écritures (requête A de
  la 15-1b, `journal_entries`), lues par la lecture du groupe **dans la même requête que ses lignes** — la jointure
  `journal_entries` existe déjà dans `FIND_GROUP_SQL` (pour `entry_number`) : seules les **colonnes** s'ajoutent
  (`je.journal`, `je.description`), aucun N+1. ⚠️ `FIND_GROUP_SQL` et `LOCK_LINES_BY_KEY_SQL` alimentent le même
  `struct LineRow` (validation P3, R L-3) : la lecture détaillée a **sa propre** constante SQL et **son propre**
  `struct` de ligne ; `LineRow`, la requête verrouillante et `find_group` (appelée par les tests de la 15-1a-i)
  restent **inchangés**.
- `ownedByDocument` = un propriétaire de l'écriture de la ligne a `DocumentKind::blocks_manual_lettering()`
  (15-1b-0) — la possession au sens de R5 ; `bankTransaction` seul ne la donne pas. C'est le prédicat même de
  `first_document_owner` (15-1b-0 D3), lu sur le même lot.
- `accountNumber`, `accountName` : le compte du groupe, lus **ensemble** par une requête propre à la lecture
  détaillée (`SELECT number, name FROM accounts WHERE id = ? AND company_id = ?`) — validation P3, R L-2 = F-5 :
  `letterable_account` (publique, appelée aussi par `is_letterable_account`, que `journal_entries.rs` appelle) et
  `group_account_number` ne lisent pas le nom et restent **inchangées**.
- **Une transaction de lecture** (validation P3, F-2) : la lecture détaillée ouvre elle-même une transaction
  (`conn.begin()`, lectures, `rollback`) où se lisent les lignes, le compte, les noms d'exercice, `document_owners`
  et la règle des périodes — un seul instantané, comme `open_items` (15-1b AC1), sous l'isolation par défaut
  d'InnoDB (`REPEATABLE READ`, que Kesh ne configure pas — réserve écrite au doc-comment comme à la 15-1b). La
  prévision reste indicative au regard du `DELETE`, mais cohérente en elle-même.
- **`manualDissolutionBlockedBy`** = le **premier** refus que la dissolution en mode `Manual` rendrait, **dans son
  ordre** (refus 1, 2, 3 de `dissolve_group_in_tx`, `letterings.rs` ; « Vérifié et confirmé » du rapport R de la
  validation P2) : `LETTERING_IS_DOCUMENT` si l'origine est `document` ; sinon
  `LETTERING_LINE_OWNED_BY_DOCUMENT` si l'origine est `reversal` et qu'une ligne a `ownedByDocument` ; sinon
  `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` si aucune ligne n'est `inOpenPeriod` ; sinon `null`. La lettrabilité du
  compte **n'entre pas** dans la prévision (C104 : la dissolution ne l'exige pas).
- ⛔ **L'ordre vit une fois, l'évaluation reste paresseuse, les détails restent ceux d'aujourd'hui** (validation P2,
  R M-1 = F2-1 ; **C-15-1c-15**). Une fonction **pure** de `kesh-db` (`letterings.rs`) :

  ```rust
  /// Le motif d'un refus de délettrage manuel — son code est celui de `DbError::error_code()`.
  pub enum ManualDissolutionBlocker { IsDocument, LineOwnedByDocument, AllLinesInClosedPeriods }
  impl ManualDissolutionBlocker { pub fn code(self) -> &'static str; }

  /// Une étape de la décision, dans l'ordre des refus 1, 2, 3. `None` = fait encore inconnu.
  pub enum DissolutionStep { Refuse(ManualDissolutionBlocker), NeedOwnership, NeedPeriod, Allowed }
  pub fn manual_dissolution_step(origin: Origin, any_owned: Option<bool>, any_in_open_period: Option<bool>)
      -> DissolutionStep;
  ```

  Ordre : `Document` → `Refuse(IsDocument)` sans rien demander ; `Reversal` et `any_owned` inconnu →
  `NeedOwnership` ; `Reversal` et `Some(true)` → `Refuse(LineOwnedByDocument)` ; puis période inconnue →
  `NeedPeriod` ; `Some(false)` → `Refuse(AllLinesInClosedPeriods)` ; `Some(true)` → `Allowed`. `Manual` ne demande
  **jamais** la propriété.
  - **La dissolution** (`Mode::Manual`, après la prise des verrous d'exercice, inchangée) appelle l'étape, ne lit
    la propriété (`first_document_owner`) **que** sur `NeedOwnership` et la borne (`books_locked_through`) **que**
    sur `NeedPeriod` — exactement la séquence de lectures d'aujourd'hui. Sur `Refuse(LineOwnedByDocument)` elle
    rend l'erreur **construite par `first_document_owner`**, avec `blocker`, `document_id`, `document_label`
    **inchangés** (rendus en `details.documentId` / `details.documentNumber` et en suffixe du message,
    `kesh-api/src/errors.rs`, `entry_document_refusal_response` / `refusal_409`) ; sur les deux autres, la
    variante sans champ (`LetteringIsDocument`, `LetteringAllLinesInClosedPeriods`). Le mode `System` n'appelle pas
    l'étape (il n'a ni refus 1 ni refus 2, inchangé).
  - **La lecture** calcule les deux faits pour toutes les lignes, **sans verrou**, et appelle l'étape avec `Some`
    et `Some` : elle ne reçoit jamais `Need*` ; si elle en recevait un, elle rend `DbError::Invariant` (jamais
    `unreachable!`, `CLAUDE.md` § « Garde-fou défensif »). Elle sérialise `blocker.code()`. La prévision est
    **indicative** (lue sans verrou) ; le `DELETE` fait autorité.
  - **Tests existants inchangés et verts** : c'est la preuve que la refonte ne change aucun comportement livré —
    ordre, codes, `details`, message suffixé, séquence de lectures.
- ⛔ **Inchangés** : `LetteringGroup` / `LetteringLine` (le type de la primitive), la réponse **201** du `POST` et
  sa forme (l'enrichissement n'y est pas calculé — C-15-1c-2 : l'écran recharge la liste après un lettrage), la
  réponse du `DELETE` (204 ou ses refus), les `details` d'audit `lettering.created` / `lettering.removed`
  (`audit_details`, champ par champ), le 404 indiscernable (autre société, clé ou code inexistant), le rôle
  (Consultation et plus, clé d'API en lecture).
- Aucune migration, aucun code d'erreur neuf, aucune clé i18n côté serveur.

**AC16 — Documentation de l'enrichissement** (`docs/api-external.md`, section « Lettrer des lignes ») : le
paragraphe de `GET /api/v1/letterings/{key}` — aujourd'hui « Réponse `200`, même forme, `origin` réel » — décrit
les champs d'AC15, dit que `manualDissolutionBlockedBy` est **indicatif** (lu sans verrou ; le `DELETE` fait
autorité ; la lettrabilité du compte n'y entre pas) et que le `POST` garde sa réponse. La mention « *(Depuis la
v0.13.0 ; l'écran viendra.)* » **reste** : elle est retirée par la 15-1c-ii (AC17), quand l'écran existe. Tout
compte cité en exemple existe dans un plan livré (G4-bis).

**AC18 — CHANGELOG et README, dans la même PR que le code** (validation P2, F2-L8, R L-6 ; C-15-1c-21).

- `CHANGELOG.md`, `[0.13.0]`, sous *Modifié* : une entrée propre sur le patron « ⚠️ Changement de contrat pour une
  intégration par clé d'API … » — `GET /api/v1/letterings/{key}` porte `accountNumber`, `accountName`,
  `manualDissolutionBlockedBy`, et par ligne `journal`, `description`, `document`, `ownedByDocument`,
  `inOpenPeriod` ; changement **additif**, le `POST` inchangé. La 15-1c-ii ne la réécrit pas (elle ne fond que les
  entrées *Ajouté*, AC14).
- `README.md`, *Feuille de route*, ligne v0.13.0 : la 15-1c-0 passe de « À venir » à « Livré sur `main` ».

## Tasks

- [ ] **T0** — Rebaser sur `main` après le merge de la 15-1b ; relever au code livré : le nom et la forme de la
      requête B et de sa part « pièce et période », le corps de la dissolution (`dissolve_group_inner` de la
      15-1a2-i, où vivent les refus 1 à 3 — validation P2, F2-L6), `first_document_owner` réécrite par la 15-1b-0,
      `FIND_GROUP_SQL` ; **inventaire des tests qui appellent la dissolution** (validation P2, R L-8) :
      `grep -rln "dissolve_group_in_tx\|dissolve_group_inner\|/letterings/" crates --include=*.rs` — chaque fichier
      noté au Dev Agent Record, il doit rester vert **sans modification**.
- [ ] **T1** (AC15) — `kesh-db` `repositories/letterings.rs` : part « pièce et période » factorisée et réemployée
      par `open_items` ; `ManualDissolutionBlocker`, `DissolutionStep`, `manual_dissolution_step` ; la dissolution
      réécrite sur l'étape (lectures paresseuses, détails inchangés) ; lecture détaillée d'un groupe
      (`find_group_detail`, ou nom relevé au T0 ; sa constante SQL, son `struct` de ligne et sa transaction de lecture
      propres ; `find_group`, `letterable_account`, `group_account_number` inchangées). `kesh-api` `routes/letterings.rs` : `LetteringDetailResponse` pour
      le seul `GET` ; le `POST` inchangé.
- [ ] **T2** (AC16) — `docs/api-external.md`, paragraphe du `GET`.
- [ ] **T3** (AC18) — `CHANGELOG.md` (*Modifié*), `README.md` (ligne v0.13.0).
- [ ] **T4** — Tests (liste ci-dessous) ; gate backend **complet** à chaque passe (`letterings.rs` est un
      repository, § « Exception `kesh-db` ») ; E2E complet au dernier commit de code (D7 — aucun écran neuf, mais la
      suite tourne : `CLAUDE.md` § « E2E »).

**Tests de T4** — un par ligne, rattaché à son critère *(Rust : `kesh-db`, binaire `letterings.rs` des tests de
dépôt ; `kesh-api`, binaire des routes du lettrage — noms relevés au T0)* :

1. AC15 — groupe `manual` : chaque ligne porte `journal`, `description`, `document = null`,
   `ownedByDocument = false`, `inOpenPeriod` ; `manualDissolutionBlockedBy = null` ; **ensemble exact des clés** de
   l'objet et de chaque ligne (égalité d'ensembles, pas seulement présence).
2. AC15 — groupe `document` (suppose la 15-1a2-i) : `document` de la créance (`invoice`) et du règlement
   (`settlement`, `invoiceNumber`) ; `manualDissolutionBlockedBy = LETTERING_IS_DOCUMENT`.
3. AC15 — groupe `reversal` dont une ligne reste **possédée** par sa pièce, produit par le **geste réel** :
   `supplier_invoices::cancel` d'une facture fournisseur validée non payée lettre `reversal` l'achat et son miroir,
   l'achat restant `OwnedBySupplierInvoice` (montage du test livré
   `supplier_invoice_cancel_letters_a_pair_that_cannot_be_dissolved_by_hand`,
   `kesh-db/tests/supplier_invoices_repository.rs` ; maintenu par la 15-1a2-ii) — validation P2, R L-7 = F2-L1 :
   aucun montage en SQL brut. Attendu : `ownedByDocument` vrai sur la ligne d'achat,
   `LETTERING_LINE_OWNED_BY_DOCUMENT`. Dans le même test, le cas contraire : la paire posée par l'annulation d'un
   **règlement client** (sa ligne `invoice_settlements` est retirée, la paire n'est plus possédée — 15-1b « Pour la
   15-1c » point 10) : `ownedByDocument` faux, prévision nulle.
4. AC15 — groupe dont toutes les lignes sont en période close → `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` ; groupe à
   cheval → `null`.
5. AC15 — **la prévision égale la dissolution** : pour chacun des cas 1 à 4, `DELETE` rend exactement le code prévu
   (ou 204 quand la prévision est nulle) — test de table ; pour le cas possédé (test 3), le `DELETE` rend **les
   mêmes `details`** (`documentId`, `documentNumber`) et le même message suffixé qu'avant la refonte. Et
   `manual_dissolution_step` en test unitaire : les **douze** combinaisons à faits connus (3 origines × possession ×
   période) ; les étapes `Need*` — `Document` n'en demande aucune, `Reversal` demande la propriété **avant** la
   période, `Manual` ne demande jamais la propriété ; `ManualDissolutionBlocker::code()` égale
   `DbError::error_code()` de la variante correspondante, pour les trois.
6. AC15 — **même source que la vue** : pour une ligne lettrée après `X` (présente dans la vue **et** dans son
   groupe), `document`, `inOpenPeriod`, `journal`, `description` sont **égaux** dans les deux réponses.
7. AC15 — compte du groupe **devenu non lettrable** par le geste réel (retypage d'un compte mouvementé, `accounts::update` avec `confirm_retype`, C104) :
   `GET` 200, `manualDissolutionBlockedBy` nul (la lettrabilité n'y entre pas), `DELETE` 204.
8. AC15 — inchangés : réponse 201 du `POST` — **test neuf** (validation P2, R L-2 = F2-L4 : les tests livrés de la
   15-1a-i ne vérifient que la **présence** de champs, `l.get(champ).is_some()`, et resteraient verts si
   l'enrichissement fuyait dans le `POST`) : **ensemble exact** des clés de l'objet et des lignes, celles de la
   15-1a-i ; `details` d'audit (tests existants verts sans modification) ; 404 d'une autre société et d'un code
   inexistant, indiscernables ; clé d'API en lecture et rôle Consultation admis.
9. AC16, AC18 — contrôles documentaires exécutés, sortie au Dev Agent Record :
   `grep -n "manualDissolutionBlockedBy" docs/api-external.md CHANGELOG.md` (présent aux deux),
   le paragraphe du `GET` relu en entier — repéré par `grep -nF 'GET /api/v1/letterings/{key}' docs/api-external.md`
   (chaîne fixe ; retenir la ligne qui ouvre le paragraphe) puis `sed -n` de ce paragraphe — et il ne dit plus « même forme » (validation P3, R L-10 : le motif a trois
   occurrences dans le fichier, une seule visée) ; ligne v0.13.0 du README relue.

*Tests existants à relire* : l'inventaire du T0 (tout test de la dissolution, de la 15-1a-i, de la 15-1a-ii, des
15-1a2-*, de la 15-1b-0 — verts **sans modification**) ; les tests de la 15-1b sur `open_items` (verts sans
modification après la factorisation) ; `letterings_lexical.rs` (il ne contraint pas l'ordre des refus, mais exige
`check_rows_affected` dans chaque primitive : la refonte n'y touche pas).

## Dev Notes

- **Modules — aux deux grains** (C-15-1a2-21) : crates et paquets — `kesh-db`, `kesh-api` = **2** ; modules de
  premier niveau — `kesh-db/repositories/letterings`, `kesh-api/routes/letterings` (deux de logique), plus
  `docs/api-external.md`, `CHANGELOG.md`, `README.md` (trois supports de texte) : **5 au grain fin**, au seuil sans
  le franchir. Aucune dérogation.
- `letterings.rs` est un **repository** : gate `kesh-db` **complet** à chaque passe qui le touche (§ « Exception
  `kesh-db` »). Le changement de forme de la dissolution est l'**axe de sécurité** de la revue : tout prompt de
  passe le nomme.
- Aucune migration (P5–P8 sans objet).
- La prévision est lue **sans verrou** ; la dissolution garde ses verrous d'exercice pris **avant** le refus 1, et
  ses lectures dans le même ordre : aucun verrou neuf, aucune lecture neuve sous verrou.
- `LetteringResponse` (le `POST`) et `LetteringDetailResponse` (le `GET`) : **composition par les champs
  communs** (`key`, `code`, `origin`, `accountId`), `lines` **propre** au détail (lignes plus larges) — jamais un
  `#[serde(flatten)]` de `LetteringResponse`, qui produirait deux clés `lines` (validation P3, F-6). Les clés JSON du
  `POST` ne bougent pas (test 8).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Création — 2026-10-09 (Opus 5.5, remédiation de la validation P2 de la 15-1c, en autonomie)

Fiche créée par extraction de la partie serveur de la 15-1c-i (décision de l'orchestrateur sur F2-3 : signal D5
levé, les MEDIUM de la P2 étant nés de la P1 ; registre **C-15-1c-14**). Reprend AC15 et AC16 de la 15-1c-i et ses
tests 1 à 8 ; ajoute AC18 (CHANGELOG *Modifié* et README dans la PR du code) et le test 9. Remédiés ici : R M-1 =
F2-1 (fonction d'ordre à étapes, détails et paresse conservés — C-15-1c-15), R L-1 = F2-L2 (douze combinaisons),
R L-2 = F2-L4 (test 8 neuf, ensembles exacts), R L-3 = F2-L3 (`journal`/`description` lus avec les lignes ; pas de
reste dû), R L-7 = F2-L1 (geste réel au test 3), R L-8 (inventaire des tests au T0), F2-L6
(`dissolve_group_inner`), F2-L8 et R L-6 (CHANGELOG et README ici — C-15-1c-21). Bilan complet par finding : Change
Log de l'index `15-1c-proposition-ecran.md`. Recompté depuis ce fichier (`grep -c '^\*\*AC[0-9]'`,
`grep -c '^- \[ \] \*\*T'`, `grep -cE '^[0-9]+\. AC'`) : **3 critères** (AC15, AC16, AC18), **5 tâches** (T0–T4),
**9 tests** numérotés.

### Validation P3 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie)

Prompt `15-1c-validate-prompt-p3.md` ; rapports `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p3-R.md` et `-F.md`.
**R : 0 MEDIUM / 10 LOW ; F : 0 MEDIUM / 9 LOW** — aucun MEDIUM+ ; les « 0 » vérifiés par l'orchestrateur (axes
déclarés exercés par les deux lentilles, recoupés au code : séquence de `dissolve_group_in_tx`, statuts des refus,
`colspan`, sites du manuel). LOW appliqués ici : R L-1 (deux pièces partagées nommées, une par crate ; unicité du code prouvée par grep), R L-2 = F-5 (nom du compte par une requête propre), R L-3 (constante SQL et `struct` de ligne propres ; `find_group` inchangée), F-2 (transaction de lecture — C-15-1c-26), F-6 (composition, pas de `flatten`), R L-10 (contrôle documentaire restreint au paragraphe du `GET`), F-8 (Status : première passe tenue en P3 de l'ensemble). Bilan complet : Change Log de l'index. Recompté : **3** critères, **5** tâches, **9** tests.

### Validation P4 ciblée — 2026-10-09 (Haiku 4.5, une lentille, commit `99280a24`) — VALIDATION CLOSE

Prompt `15-1c-validate-prompt-p4-ciblee.md` ; rapport `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p4-F.md`.
**0 MEDIUM / 4 LOW**, vérifiés par l'orchestrateur. Appliqués ici : F-1 (`journal_entries.rs` appelle
`is_letterable_account`, non `letterable_account` — `grep -n` → `journal_entries.rs:2578`), F-2 (commande du test 9
réécrite en `grep -nF`, sans accents graves imbriqués). Remédiation documentaire, aucune règle ni contrat changé : la
boucle se clôt (`CLAUDE.md` § « La passe ciblée »).
