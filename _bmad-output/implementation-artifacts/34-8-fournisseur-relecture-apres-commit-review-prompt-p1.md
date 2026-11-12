# Revue de code P1 — Story 34-8 (#938) : une facture fournisseur n'est plus annoncée en échec après son commit

Revue adversariale **complète**, trois lentilles en contexte frais : **B** (Blind Hunter), **E** (Edge Case Hunter), **A**
(Acceptance Auditor). Première passe : Sonnet ou Opus, jamais Haiku.

## Périmètre

- Worktree : `/home/gcorbaz/devel/kesh-e348`, branche `story/34-8-fournisseur-relecture-apres-commit`.
- Diff : `git -C /home/gcorbaz/devel/kesh-e348 diff 665de151..HEAD` (base `665de151`).
- Fiche : `_bmad-output/implementation-artifacts/34-8-fournisseur-relecture-apres-commit.md` (AC-1 à AC-9, mutations M1 à M6,
  Dev Agent Record).
- Choix : `_bmad-output/implementation-artifacts/e34-choix-autonomes-e343.md` (C-e343-9 à 12, 15) et
  `e34-choix-autonomes-e348.md` (C-e348-1 à 6).
- Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e348/` (rouges, mutations, gates, E2E) ; captures dans
  `/home/gcorbaz/devel/kesh-gate-logs/e348/captures/`.

## Interdits

Lecture seule. Aucune commande qui écrit, compile, exécute un test ou lance un script (`cargo`, `npm`, `npx`,
`scripts/*.sh`, `scripts/prepare-release.sh` compris), aucun `git` qui écrit (`checkout`, `commit`, `stash`, `reset`…),
aucune redirection (`>`, `>>`, `tee`) ni fichier temporaire. Permis : `git diff`, `git show`, `git log`, `git grep`,
`/usr/bin/grep`, `sed -n`, `cat`, `pdftotext … -` vers la sortie standard. Toute affirmation de présence ou d'absence
s'appuie sur une sortie `grep -nF` copiée dans le rapport. Le seul fichier écrit est ton rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e348-review-p1-<B|E|A>.md` selon ta lentille : findings numérotés `<lentille>-n`, sévérité
(CRITICAL, HIGH, MEDIUM, LOW), preuve (fichier:ligne, extrait), correction proposée. Puis **la liste des axes exercés et
celle des axes non exercés** : une passe qui ne la donne pas ne compte pas.

## Axes communs

1. **`commit_with_view`** (`crates/kesh-db/src/repositories/supplier_invoices.rs`) : rollback sur `Err` et sur vue absente ou
   en échec, commit après la lecture, `view_id` appelée seulement sur `Ok`, erreur générique `E: From<DbError>`. Un chemin
   qui commit sans vue, ou qui rend une vue sans commit ?
2. **`get_settlement_view_in_tx`** : aucune lecture verrouillante ajoutée ; l'instantané de la transaction d'écriture
   (C-e343-10) écrit au doc-comment ; `get_settlement_view` la réemploie (DRY) et garde son comportement pour le service
   en lecture seule (`services/mod.rs`, « soixante éléments » : le décompte est-il juste ?).
3. **Les cinq enveloppes** (`validate`, `unvalidate`, `pay`, `cancel`, `cancel_settlement`) et **les quatre `_once`** des
   routes : plus aucun `tx.commit()` direct ; les formes `_in_tx` inchangées (`git diff` sur `pay_in_tx`, `cancel_in_tx`,
   `cancel_settlement_in_tx`, `validate_in_tx`, `unvalidate_in_tx`) ; `SupplierSettlementCancellation` inchangée,
   `SupplierSettlementCancelled` neuve.
4. **Les neuf handlers** : rien d'attendu après l'écriture ; `load_with_settlement_cancellation` retirée, et plus aucune
   mention (`git grep -n load_with_settlement_cancellation`). Le rejeu sur interblocage (`retry_on_deadlock`,
   `retry_app_on_deadlock`) rend-il bien la vue de la tentative qui a réussi (AC-7) ?
5. **La garde de structure** (`crates/kesh-api/tests/supplier_invoice_response_e2e.rs`, module
   `crates/kesh-api/tests/common/autour_appel.rs`) : mord-elle ? La sonde positive passe-t-elle par le même découpage ?
   La garde du client (33-6, `invoice_settlement_response_e2e.rs`) est-elle inchangée dans ses assertions ? Les limites
   écrites sont-elles reprises ?
6. **Les égalités AC-4** : un test par geste, la réponse égale le `GET` entier ; un geste sans test ?
7. **L'écran** (`frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`) : les sept gestes, `cycleOutcome`, les deux
   avis, `reloadAfterRefusal` qui rend un booléen, `attach` (trois branches, C-e348-2, C-e348-3), `detach`, un 4xx inchangé.
   Un `return` dans un `catch` qui saute un `finally` attendu ? Un état qui reste affiché à tort ?
8. **Les tests Vitest** (`supplier-unknown-outcome-page.test.ts`) et les deux tests modifiés de
   `supplier-document-page.test.ts` (C-e348-4) : affaiblissement d'une assertion existante ?
9. **Textes** : les quatre locales de `supplier-invoices-unknown-outcome` et `…-reread-failed`, replis identiques au fr-CH ;
   `docs/api-external.md:888` ; les doc-comments C-30-2-42 et C-30-2-52 récrits ; `unknown-outcome.ts` ; CHANGELOG.
10. **Le manuel** : `docs/manual/fr/user-manual.tex`, *Régler une facture fournisseur* (`sec:regler-facture-fournisseur`),
    paragraphe « Si la réponse se perd ». **Contrôle aussi le PDF** `docs/manual/fr/user-manual.pdf`, aplati :
    `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o "Si la réponse se perd[^.]*\. [^.]*\."`.
    Le manuel dit-il vrai du code (sept gestes, refus d'un second envoi, rechargement) ?
11. **Les captures** de `/home/gcorbaz/devel/kesh-gate-logs/e348/captures/` : l'encadré se voit-il, en thème clair ?
12. **Les déclarations** du Dev Agent Record et du Change Log : décomptes recomptés, gates déclarés tels qu'ils ont tourné.

## Lentilles

- **B (Blind Hunter)** : le diff seul, sans la fiche. Défauts de code, de logique, de concurrence, d'erreurs avalées.
- **E (Edge Case Hunter)** : chaque branche et chaque bord — vue `None`, rollback qui échoue, rejeu, réponse perdue puis
  relecture qui échoue, document joint ailleurs pendant la relecture, double clic, facture d'une autre société.
- **A (Acceptance Auditor)** : chaque AC de la fiche contre le code et les tests, les mutations M1 à M6 et leurs journaux,
  les choix consignés, le manuel et son PDF.
