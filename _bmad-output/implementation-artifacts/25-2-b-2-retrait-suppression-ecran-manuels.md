# Story 25.2-b-2 : La suppression ne traite plus que les brouillons — l'écran et les manuels

Status: ready-for-dev

**Issue : [#440]**, qu'elle **ferme** : `closes #440` dans le **titre ET le corps** de sa PR.
⚠️ Elle **ne rouvre pas [#219]**, fermée : la story en remplace le chemin, pas la décision. Et elle
**ne ferme pas [#381]**, close par la 25-2-c.

⛔ **Issue du DÉCOUPAGE de la 25-2-b** (arbitrage de Guy, 2026-09-21). **La fiche mère
`25-2-b-devalidation-facture.md` reste la source des faits établis** par ses trois passes de
validation.

**Prérequis** : la **25-2-b-1** mergée — elle pose `unvalidate`, sans quoi retirer la suppression
d'une facture validée laisserait les utilisateurs sans aucun chemin de correction.

## Story

En tant que **personne qui facture**,
je veux que **« supprimer » ne concerne plus qu'un brouillon**, et que l'écran m'offre
**« dévalider »** sur une facture validée,
afin que **la destruction d'une écriture ne soit plus un effet de bord caché du verbe
« supprimer »**.

## Acceptance Criteria

1. **`invoices::delete` ne traite plus que les brouillons.** Sa branche `status == "validated"` et
   ses trois gardes disparaissent — elles vivent désormais dans `unvalidate`. Une facture validée y
   est refusée sous un code propre — **`INVOICE_MUST_BE_UNVALIDATED_FIRST`, `409`** — dont le
   message **oriente vers la dévalidation**, et non sous le générique `ILLEGAL_STATE_TRANSITION` ;
   un test neuf l'exerce : la branche retirée ne doit pas redevenir
   un chemin silencieux.

2. **`enforce_immutability = false` n'a plus qu'un appelant de production** — `unvalidate`. Le
   drapeau **reste dans `delete_in_tx`**, jamais chez l'appelant : une garde posée chez l'appelant
   laisserait la fonction nue pour le suivant. ⚠️ **Les trois doc-comments qui nomment
   `invoices::delete` comme seul appelant** (`journal_entries.rs:943`, `:982` ;
   `invoices.rs:1339`) sont réécrits **par la 25-2-b-1**, qui les périme en créant le second
   appelant. Ici, seule la phrase qui décrit le chemin retiré se met à jour.

3. **HUIT tests existants se réécrivent contre `unvalidate`** — ils ne se suppriment pas, sous peine
   d'emporter la couverture des gardes :
   - `crates/kesh-db/src/repositories/invoices.rs` : `test_delete_validated_unpaid_open_fy_removes_invoice_and_je`,
     `test_delete_validated_paid_is_rejected`, `test_delete_validated_in_closed_fy_is_rejected`,
     `test_delete_validated_credited_by_avoir_is_rejected`,
     `test_delete_validated_with_reminders_is_rejected` ;
   - `crates/kesh-api/tests/invoice_delete_e2e.rs` : `delete_validated_as_admin_returns_204`, et
     ⛔ **`delete_validated_in_locked_period_returns_400_period_locked`**, ajouté par la
     **25-2-b-zero** (#443) — relevé en passe 3, il porte le motif « période verrouillée » par le
     chemin de suppression ; il se réécrit par la dévalidation.
   - Et le Playwright `frontend/tests/e2e/invoices.spec.ts:333-367` (« fiche fantôme »), qui
     supprime une facture **validée** par l'API : il dévalide d'abord.
   ⚠️ **HUIT au total — 5 + 2 + 1 —, et ce chiffre a été faux trois fois.** La fiche mère disait
   « cinq » (elle ignorait les deux E2E backend), cette fiche a d'abord titré « six » en en
   énumérant huit, et sa propre note de correction disait « sept ». Recompté depuis la source en
   passe 1 de validation : `grep -c "async fn test_delete_validated" invoices.rs` → **5** ;
   `invoice_delete_e2e.rs` → **2** (`:203`, `:246`) ; Playwright → **1**. ⛔ *Corriger un décompte
   et écrire juste sont deux gestes distincts.* ⚠️ `delete_requires_auth_returns_401` et
   `delete_as_comptable_returns_403` sont **exclus à bon droit** : ils échouent en amont, sur l'auth
   et le rôle, sans atteindre la branche retirée.

4. **L'écran.** Sur une facture **validée**, le bouton « Supprimer » devient **« Dévalider »**, et
   dit ce qui va se passer : l'écriture comptable est supprimée, la facture repasse en brouillon,
   son numéro est conservé. Les refus de l'AC 3 de la fiche mère s'affichent **en nommant leur
   motif**. Friction : une confirmation simple — **pas** de numéro à retaper, la facture restant en
   place.
   - ⚠️ **Résidu à retirer** : la branche `invoice?.status === 'validated'` de la modale
     « retaper le numéro » (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:919-945` — le bloc `{#if invoice?.status === 'validated'}` en entier —, et `:958`)
     devient du code mort.
   - ⚠️ **La garde actuelle `isAdmin && !invoice.paidAt` (`:729`) sous-couvre le motif 1** : une
     facture *partiellement* réglée n'a pas de `paid_at`. L'écran lit le résiduel, ou assume le
     refus serveur — au choix, mais écrit.
   - ⚠️ **L'asymétrie des deux sorties s'affiche** (arbitrage de Guy, 2026-09-19) : dévaliser est
     ouvert au **Comptable**, mais **effacer** reste réservé à l'**Administrateur** (`admin_routes`,
     décision de #219, fermée aux clés API). Le dire au Comptable plutôt que de lui laisser
     découvrir un `403`.

5. **Effacer un brouillon numéroté laisse un trou dans la séquence des FACTURES** — le compteur ne
   redescend pas, et c'est voulu. La confirmation de suppression d'un brouillon **qui porte un
   numéro** le dit ; celle d'un brouillon jamais validé reste inchangée.

6. **E2E : les deux cycles, bout à bout.** Dévaliser puis effacer ; dévaliser, corriger, revalider
   — et vérifier que le **numéro de facture est le même** après revalidation.

7. **Les manuels**, sites établis par les passes 1 à 3 :
   - `docs/manual/fr/user-manual.tex` — §`sec:suppression-facture` (l. 1001-1022) : **refonte** ;
     l. 464-465 : la dévalidation **entre dans l'énumération** des chemins que le verrou ferme ;
     l. 502 et 618, l. 538, l. 778, l. 834, et l. **968** (« une suppression définitive encadrée est également possible » — l. 966 est l'en-tête `\subsection{Avoirs et notes de crédit}`) : relus contre le cycle neuf ;
   - `docs/manual/fr/admin-manual.tex` — l. 1759 (routes d'administration) ; l. 1799 et
     `README.md:218` (« ni modifiable ni supprimable […] sans exception », **déjà faux**) ;
     ⛔ l. 1957 : **tout le paragraphe se réécrit** — il repose sur la prémisse qu'un administrateur
     peut supprimer une facture qui a des rappels, fausse déjà sur `main`.
   - PDF régénérés et **contrôlés à plat** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`). ⚠️ Le PDF
     du manuel d'administration n'a encore été aplati par aucune passe.

8. **i18n.** Les libellés de l'écran dans les quatre locales, et le message
   `journal-entries-reverse-blocked-invoice` (« corrigez-la par un avoir ») — quatre locales et son
   repli dans `crates/kesh-api/src/errors.rs` — nomme désormais aussi la dévalidation.

9. **`docs/api-external.md`** : la note ² (l. 217) ne décrit plus `DELETE /invoices/{id}` comme une
   « suppression définitive » — la route ne supprime plus que des brouillons.

10. **`CHANGELOG.md`**, section « Non publié » : l'entrée du cycle complet, côté utilisateur, **et
    le compteur de libellés d'audit** — « 123 libellés » devient **124**, la 25-2-b-1 ayant ajouté
    `invoice.unvalidated`. ⚠️ Elle le rend faux et ne touche pas ce fichier ; c'est donc ici qu'il se
    corrige. Recompter depuis `crates/kesh-api/src/audit_labels.rs` (types d'entité + actions +
    types d'auteur), ne pas incrémenter de confiance.

11. **Gate complet** — la story touche `kesh-db`, le frontend et les manuels : gate backend complet,
    gate frontend, E2E complète avant le push.

## Tasks / Subtasks

- [ ] **T1 — Dépôt** (AC 1, 2, 3) — retrait de la branche, réécriture des **huit** sites de test.
- [ ] **T2 — Écran** (AC 4, 5, 8).
- [ ] **T3 — E2E des deux cycles** (AC 6).
- [ ] **T4 — Manuels, PDF, `api-external.md`, `CHANGELOG`** (AC 7, 9, 10).
- [ ] **T5 — Gates complets** (AC 11), PR avec `closes #440`.

## Dev Notes

### Ce que cette story ne fait pas

Elle n'ajoute aucune garde métier : toutes vivent déjà dans `unvalidate` (25-2-b-1) et dans
`delete_in_tx` (25-2-b-zero). Si une garde manque au moment du développement, c'est un défaut de la
25-2-b-1 — le corriger là-bas, pas ici.

### References

- `_bmad-output/implementation-artifacts/25-2-b-devalidation-facture.md` — la fiche mère.
- `_bmad-output/implementation-artifacts/25-2-b-1-devalidation-depot-api.md` — le prérequis.
- [Source: crates/kesh-db/src/repositories/invoices.rs] — `delete` et ses trois gardes.
- [Source: frontend/src/routes/(app)/invoices/[id]/+page.svelte] — le bouton et sa modale.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-21 | spec | Story née du **découpage** de la 25-2-b (arbitrage de Guy). Elle hérite des trois passes de validation de la fiche mère. ⚠️ Son AC 3 annonçait corriger un décompte de la mère (cinq) et s'est trompé à son tour, à trois reprises dans le même paragraphe ; recompté en passe 1 de validation : **huit**. |
| 2026-09-21 | validate P1 | **Passe 1, une lentille Sonnet**, prompt versionné `25-2-b-2-validate-prompt-p1.md`, sept axes exercés. **1 HIGH, 3 MEDIUM, 1 LOW**, tous vérifiés depuis la source avant d'être retenus. ⛔ **Le HIGH est une faute de décompte DANS la phrase qui corrigeait un décompte** : titre « SIX », énumération de **huit**, note finale « Sept ». Recompté : **8** (5 + 2 + 1). MEDIUM : le refus de `delete` sur une facture validée n'avait pas de code nommé (`INVOICE_MUST_BE_UNVALIDATED_FIRST`, `409`) ; deux plages de lignes de `journal_entries.rs` s'arrêtaient **juste avant** la phrase visée (943 et 982) ; la plage du résidu d'écran désignait le gestionnaire du dialogue et non le bloc mort (`:919-945`). LOW : `user-manual.tex:966` est l'en-tête de sous-section, la phrase est à **968**. ⚠️ Ces trois citations étaient **héritées mot pour mot de la fiche mère**, et aucune de ses trois passes ne les avait rouvertes — *une citation recopiée n'est pas une citation vérifiée*. Vérifié sans rien trouver : l'asymétrie des sorties, les cinq citations de manuel et d'`api-external.md`, les PDF aplatis des deux manuels FR, l'ordre des deux filles, le périmètre (4 modules). |
