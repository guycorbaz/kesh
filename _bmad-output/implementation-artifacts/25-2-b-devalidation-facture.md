# Story 25.2-b : Dévalider une facture — pour l'effacer, ou pour la corriger

Status: draft — ⛔ **une issue GitHub reste à ouvrir** (cf. § *Traçage*)

## Story

En tant que **personne qui facture**,
je veux pouvoir **repasser une facture validée en brouillon**, puis soit l'**effacer**, soit la
**corriger et la revalider**,
afin de **réparer une erreur de facturation sans polluer les livres d'un avoir** qui ne correspond à
aucune réalité commerciale.

**Précédent cité par Guy (2026-09-16) : bexio se comporte ainsi.** *(Consigné comme référence de
comportement attendu ; la sémantique exacte de bexio n'a pas été vérifiée ici et ne doit pas être
invoquée comme spécification.)*

## L'arbitrage, et ce qu'il remplace

Guy, 2026-09-16, en deux temps :

1. d'abord « fermer le chemin résiduel de #381 » — l'écriture d'une facture ne doit plus disparaître ;
2. puis, la contre-passation s'étant révélée **inapplicable** (`ReversalBlocker::OwnedByInvoice`,
   `crates/kesh-db/src/errors.rs:63-64` : l'Epic 24 a décidé que l'écriture d'une facture se corrige
   par l'**avoir**, jamais par une extourne) : **« pour supprimer une facture validée, il faut
   pouvoir la repasser en brouillon, puis l'effacer »**, et — demandé explicitement —
   **« dévalider-effacer *et* dévalider-corriger-revalider »**.

⚠️ **Ce que cela ne fait pas, et il faut le dire net** : en deux gestes ou en un, **l'écriture est
supprimée de la même façon**. Cette story ne referme pas #381 ; elle rend la destruction **explicite
et auditable** au lieu d'être un effet de bord caché du verbe « supprimer ». La réattribution muette
des numéros est fermée par la story **25-2-c**, séparément.

**Ce qu'elle remplace** : la suppression directe d'une facture validée (#219, décision produit de Guy
du 2026-07-07 — *« suppression réelle, pas simple annulation »*). Le besoin est conservé ; c'est le
chemin qui change, et il gagne une étape nommée.

## L'état actuel, établi

- **Aucun chemin de retour vers `draft` n'existe.** `validate_invoice`
  (`crates/kesh-db/src/repositories/invoices.rs:1550`) est à sens unique ; les fonctions publiques du
  dépôt ne comportent aucune dévalidation.
- La contrainte `chk_invoices_validated_has_je` **anticipe déjà ce cas** en toutes lettres : *« une
  facture draft peut avoir journal_entry_id NULL (cas nominal) ou exceptionnellement non-null —
  aucun chemin applicatif actuel, mais autorisé par le CHECK »*
  (`20260417000002_invoice_validated_journal_entry_check.sql:8-13`).
- La validation écrit : `status = 'validated'`, `invoice_number` **tiré d'un compteur qui ne
  redescend jamais**, `journal_entry_id`, `version + 1`, et crée l'écriture de vente
  (`invoices.rs:1742-1845`).
- Le seul site du dépôt passant `enforce_immutability = false` à `journal_entries::delete_in_tx` est
  `invoices::delete` (`invoices.rs:1350`).

## Acceptance Criteria

1. **La transition existe.** `POST /api/v1/invoices/{id}/unvalidate`, dans la famille de
   `POST /api/v1/invoices/{id}/validate` (`crates/kesh-api/src/lib.rs:486`). Elle repasse la facture
   en `draft`, met `journal_entry_id` à `NULL`, **puis** supprime l'écriture — dans cet ordre, la FK
   `journal_entry_id … ON DELETE RESTRICT` l'imposant.

2. **Le numéro de facture est CONSERVÉ.** `invoice_number` n'est **pas** effacé par la dévalidation,
   et la revalidation le **reprend tel quel** sans tirer du compteur. ⛔ **Sans ce critère, la story
   transporte la maladie de #381 sur la numérotation des factures** : le compteur ne redescendant
   jamais, chaque cycle dévalider/revalider brûlerait un numéro et creuserait un trou dans la
   séquence des factures.

3. **Les empêchements forment une liste CLOSE, et chacun se justifie.** La dévalidation est refusée,
   avec un code distinct par motif, si la facture ou son écriture est dans l'un de ces états —
   inventaire établi depuis le schéma (tout ce qui référence `invoices(id)` ou l'écriture), et non
   depuis une énumération de cas imaginés :

   | Empêchement | D'où il vient | Pourquoi |
   |---|---|---|
   | facture **payée**, même partiellement | `invoice_settlements` (CASCADE), `paid_at` | un règlement pointerait dans le vide |
   | facture **créditée** par un avoir | `credit_notes` (RESTRICT) | l'avoir est déjà la contre-passation |
   | facture avec **historique de rappels** | `invoice_reminders` (CASCADE) | effacerait la preuve de recouvrement (#260) |
   | facture **envoyée au client** | `emailed_at` | le client détient un document que les livres ne porteraient plus |
   | **exercice clos** | `fiscal_years.status` | CO 958f — déjà appliqué par `delete_in_tx` |
   | écriture **rapprochée** d'une transaction bancaire | `bank_transactions.matched_entry_id` | désynchroniserait le rapprochement en silence |
   | écriture **contre-passée** | `journal_entries.reverses_entry_id` | l'extourne resterait sans origine |

   ⚠️ **Les quatre derniers ne sont PAS gardés aujourd'hui par `invoices::delete`** : il n'en applique
   que trois (payée, créditée, rappels) et délègue l'exercice clos. La dévalidation les ajoute — et
   c'est un durcissement, pas une régression.

   ⛔ **`emailed_at` : REFUS SEC, non levable par confirmation.** Arbitrage de Guy, 2026-09-16 :
   *« on ne doit pas pouvoir modifier une facture créée et envoyée à un client »*. C'est le seul des
   sept empêchements qui ne se déduise pas du schéma — les six autres protègent une donnée interne,
   celui-ci protège un document **sorti de Kesh**, que le client détient. Une fois la facture partie,
   le chemin de correction redevient l'**avoir**, et c'est cohérent : l'avoir existe précisément pour
   corriger ce qu'un tiers a déjà reçu.

   ⚠️ **La garde n'attrape que ce que Kesh sait avoir envoyé.** `emailed_at` est renseigné par
   `mark_emailed` ; une facture dont le PDF a été téléchargé puis transmis à la main ne laisse
   **aucune trace**, et restera dévalidable. La limite est structurelle, pas réparable ici — elle
   s'écrit, elle ne se comble pas.

4. **Deux sorties, et elles sont symétriques.**
   - **effacer** : la facture redevenue `draft` se supprime par le chemin brouillon **existant**, sans
     garde nouvelle — il n'y a plus d'écriture à protéger ;
   - **corriger et revalider** : la facture `draft` se modifie par `PUT /api/v1/invoices/{id}`
     **existant**, puis se revalide par la route de validation existante, qui recrée une écriture
     **neuve** portant un numéro d'écriture **neuf**. C'est correct et voulu : ce n'est pas la même
     écriture.

5. **`invoices::delete` ne traite plus que les brouillons.** Sa branche `status == "validated"` et
   ses trois gardes disparaissent, déplacées vers la dévalidation. ⛔ **Conséquence à vérifier comme
   un critère** : `enforce_immutability = false` n'a plus alors qu'un appelant — le nouveau — et le
   drapeau doit rester **dans `delete_in_tx`**, jamais chez l'appelant, pour la raison déjà écrite à
   `journal_entries.rs` : une garde posée chez l'appelant laisserait la fonction nue pour le suivant.

6. **L'audit nomme le geste.** Code d'action neuf `invoice.unvalidated`, avec un instantané de l'état
   avant (numéro, total, écriture supprimée), en plus du `journal_entry.deleted` que
   `delete_in_tx` journalise déjà. Même dépendance au registre `audit_labels.rs` que la 25-2-a
   (AC 6 de celle-ci) : **le fichier n'existe que sur la branche de la PR #439**, non mergée —
   constater son état au démarrage et appliquer la branche correspondante.

7. **Les messages sont traduits** dans les quatre locales, convention `error-*`.

8. **Le verrou optimiste s'applique.** La dévalidation prend `version` et rend 409 en cas de conflit.

9. **L'écran.** Le bouton « supprimer » d'une facture validée est remplacé par « dévalider », qui
   explique ce qui va se passer : l'écriture comptable sera supprimée, la facture repassera en
   brouillon, et le numéro sera conservé. Les refus de l'AC 3 s'affichent en nommant leur motif.

10. **Les manuels.** Le manuel utilisateur décrit le cycle et ses empêchements ; le passage décrivant
    la suppression d'une facture validée est rectifié. PDF régénéré et **contrôlé à plat**.

11. **`docs/api-external.md`** porte la route neuve et ses codes d'erreur.

## Tasks / Subtasks

- [ ] **T0 — Traçage** (cf. § *Traçage*) — ouvrir l'issue, reporter son numéro ici et dans le titre.
- [ ] **T1 — Dépôt : la dévalidation** (AC 1, 2, 3, 5, 8)
  - [ ] `invoices::unvalidate`, transaction unique : verrou, inventaire des empêchements, `NULL` sur
        `journal_entry_id`, `status = 'draft'`, suppression de l'écriture, audit.
  - [ ] Retrait de la branche `validated` de `invoices::delete`.
  - [ ] Tests : un par empêchement de l'AC 3, plus le chemin nominal, plus le conflit de version.
- [ ] **T2 — Dépôt : la revalidation reprend le numéro** (AC 2, 4)
  - [ ] `validate_invoice` ne tire du compteur **que** si `invoice_number` est absent.
  - [ ] ⛔ Test décisif : valider → dévalider → revalider → **le même numéro de facture**, et le
        compteur n'a pas bougé.
- [ ] **T3 — API, i18n, audit** (AC 1, 6, 7)
- [ ] **T4 — Frontend** (AC 9)
- [ ] **T5 — E2E** — les deux cycles complets, bout à bout.
- [ ] **T6 — Documentation** (AC 10, 11) + `CHANGELOG.md`.
- [ ] **T7 — Gates complets et PR.**

## Dev Notes

### Traçage — ⛔ à faire avant le développement

La dévalidation est une **fonctionnalité neuve**, pas un correctif : la règle de traçage du dépôt
impose une issue GitHub (`feature_request.yml`) **avant** le changement de scope. **Elle n'est pas
ouverte** — Guy doit le demander. #219 est fermée et décrit le chemin que cette story remplace ;
#381 appartient à la 25-2-c.

⚠️ **Le numéro se lit depuis GitHub au moment de l'attribution**, jamais depuis un fichier ou une
mémoire : rien ne contrôle l'unicité, et une collision est silencieuse.

### Ce que cette story ne fait pas

Elle ne touche **pas** à la numérotation des écritures : c'est la **25-2-c** (#381), qui devrait la
précéder — cette story rend la suppression d'écritures délibérée et fréquente, et l'intervalle entre
les deux est exactement la fenêtre où des numéros seraient réattribués en silence.

### Règle de splitting

Modules touchés : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — quatre, sous le seuil. Mais le
scope est **plus lourd que la 25-2-a** : une transition d'état neuve, sept empêchements, deux cycles
E2E. Si une passe de `validate` remonte une sévérité égale ou supérieure à la précédente, splitter
selon le critère de non-convergence.

### References

- [Source: crates/kesh-db/src/errors.rs:57-88] — `ReversalBlocker`, dont `OwnedByInvoice`.
- [Source: crates/kesh-db/src/repositories/invoices.rs:1205-1360] — `delete` et ses trois gardes.
- [Source: crates/kesh-db/src/repositories/invoices.rs:1550-1845] — `validate_invoice`.
- [Source: crates/kesh-db/migrations/20260417000002_invoice_validated_journal_entry_check.sql]
- [Source: https://github.com/guycorbaz/kesh/issues/219] — la suppression directe, fermée, que cette story remplace.
- [Source: CLAUDE.md § Issue Tracking Rule, § Review Iteration Rule]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-16 | spec | Story créée sur arbitrage de Guy, après que la contre-passation se soit révélée inapplicable (`OwnedByInvoice`). Deux cycles demandés : dévalider-effacer **et** dévalider-corriger-revalider. ⛔ Issue GitHub non ouverte. |
| 2026-09-16 | arbitrage | `emailed_at` → **refus sec**, non levable : *« on ne doit pas pouvoir modifier une facture créée et envoyée à un client »* (Guy). Réserve écrite : la garde n'attrape que ce que Kesh sait avoir envoyé. ⚠️ Fait relevé au passage, **contre l'asymétrie supposée** : les factures **reçues** ne sont pas plus faciles à modifier — `supplier_invoices` n'a **aucune** fonction de mise à jour, et son `cancel` **contre-passe** l'écriture d'achat au lieu de la supprimer. Le côté fournisseur est donc plus strict, pas plus souple. |
