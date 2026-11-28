# Story 53-5 : Solde de l'Epic 53 — trois issues P4 de dette des stories de l'epic (#1653, #1656, #1657)

Status: drafted

*Fiche rédigée le 2026-11-28 (Sonnet 5.5), worktree `kesh-53-5`, branche `story/53-5-solde-dette-epic` partie de `epic/53-ecritures-justes` à `4ac401e36`, par le workflow `bmad-create-story`, en autonomie. Epic 53 « Écritures justes » (réduit le 2026-11-28 : il se clôt avec la release des corrections P1), story de solde. **Aucune migration, aucune règle métier, aucun changement d'API.** Les trois issues sont de la dette de revue de priorité basse : aucune ne touche la comptabilité. Non validée.*

## Story

En tant que mainteneur de Kesh, je veux solder la dette que les revues des stories 53-1a et 53-3 ont laissée en issues — un raccourci clavier non affirmé par un test, un doc-comment inexact sur le mode d'un verrou, une garde lexicale dont la limite n'est pas écrite —, afin que l'Epic 53 se clôt sans autre dette de revue qu'ouverte et nommée.

## Choix du lot (plafond de trois issues)

Plafond fixé par le Project Lead le 2026-11-23 : **trois issues**, choisies par priorité (P2, P3, P4) puis ancienneté. L'orchestrateur propose #1653, #1656, #1657 ; **les trois sont retenues**, après vérification qu'aucune n'est déjà corrigée (étape 0). Aucune substitution.

## Étape 0 — prémisses relues au code (`4ac401e36`)

**P1 — #1653 reste ouverte (confirmée).** `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:895-907` : `handleKeydown` pose `e.preventDefault()` (`:898`), puis `(document.activeElement as HTMLElement | null)?.blur()` (`:902`), puis `if (pendingAssistant !== null) return;` (`:904`), puis `if (canSubmit) handleSubmit();`. Le test `JournalEntryForm.assistant.test.ts:91-97` (« Ctrl+S, modale ouverte, n’envoie rien ») affirme `createMock` non appelé et la modale encore présente ; **il n'affirme rien sur `defaultPrevented`** (`grep -n "defaultPrevented"` sur `frontend/src` ne rend aucun test de ce raccourci). Le `fireEvent.keyDown(window, …)` de `@testing-library/svelte` rend un booléen : `false` si un écouteur a appelé `preventDefault()` (c'est le retour de `dispatchEvent`).

**P2 — #1656 reste ouverte (confirmée), avec des numéros de ligne de l'issue périmés.** `crates/kesh-db/src/repositories/journal_entries.rs:1066-1068` (doc-comment de `create_opening_entry`, § « Verrous, dans l'ordre ») écrit : « Les gestes qui **modifient** un compte existant (dont le retypage, `accounts::update`, `archive_guarded`, `reactivate`) prennent `companies` en partage d'abord ». Au code actuel : `accounts::update_in_tx` prend `CompanyLock::Share` seulement si `own_fx_accounts` est `Unchanged`, sinon `CompanyLock::ForUpdate` (`accounts.rs:1219-1224`) ; `accounts::create_in_tx`, idem (`:215-218`, `ForUpdate` si `own_fx_accounts.is_some()`). `archive_guarded` (`:1908`) et `reactivate` (`:2109`) prennent `Share` sans condition. Les numéros de l'issue (`:1148-1152`, `:195-197`) ont bougé de quelques lignes : **la fiche cite ceux d'aujourd'hui**. La conclusion du doc-comment (exclusion au sentinelle `companies FOR UPDATE` de `create_opening_entry`) reste vraie : `FOR UPDATE` et `LOCK IN SHARE MODE` s'excluent. Seul le **mode décrit** est inexact. Une seule occurrence de la formule dans tout le dépôt : `grep -rn "en partage d'abord" crates docs` rend `journal_entries.rs:1067` et, hors production, la fiche 53-3 (`:374`, qui cite l'issue et ne se réécrit pas).

**P3 — #1657 reste ouverte (confirmée).** `crates/kesh-db/tests/opening_types_sous_verrou_lexical.rs:184-190` : `ordre_lecture_et_recensement` prend `corps.find(&format!("{LECTURE}("))` et `corps.find("count_by_company(")` dans le corps **masqué** de `create_opening_entry`, puis `la_lecture_des_types_precede_le_recensement` (`:200-208`) compare les deux positions. Une lecture placée sous un `if`, ou après un retour anticipé, garderait sa position textuelle sans s'exécuter sur tous les chemins. La doc du test (`:192-199`) écrit la limite des **noms** (« elle ne connaît que les noms des deux appels »), **pas** celle du **flux** (ordre textuel, non ordre d'exécution). Une occurrence de chaque appel dans le code actuel, comme dit l'issue.

## Acceptance Criteria

### Lot A — #1653 : Ctrl+S, modale ouverte, empêche l'action par défaut du navigateur

1. **AC-1.** Le test `Ctrl+S, modale ouverte, n’envoie rien` (ou un test voisin du même fichier) affirme que l'événement Ctrl+S émis modale ouverte est **annulé** : `const poursuivi = await fireEvent.keyDown(window, { key: 's', ctrlKey: true }); expect(poursuivi).toBe(false);`. Les assertions existantes (`createMock` non appelé, modale présente) sont conservées.
2. **AC-2.** Le test rougit sur **chacune** de ces deux mutations de `JournalEntryForm.svelte` (appliquées à la main, binaire/test relancé, puis restaurées ; consignées au Dev Agent Record) :
   - **M1** : supprimer `e.preventDefault();` (`:898`) → `fireEvent.keyDown` rend `true`, AC-1 échoue ;
   - **M2** : déplacer `e.preventDefault();` **après** `if (pendingAssistant !== null) return;` (`:904`) → modale ouverte, l'événement n'est plus annulé, AC-1 échoue.
3. **AC-3.** Aucun code de production du frontend ne change : le correctif est le test seul.

### Lot B — #1656 : le doc-comment de `create_opening_entry` dit le bon mode de verrou (texte pur)

4. **AC-4.** Le § « Verrous, dans l'ordre » de `create_opening_entry` (`journal_entries.rs`, doc-comment, ~`:1066-1068`) dit que les gestes qui modifient un compte existant prennent `companies` « d'abord (en partage, ou en `FOR UPDATE` quand le geste change les comptes en devise propres : `update`, `create`) », en nommant les sites vrais au code actuel (`accounts::update_in_tx`, `accounts::create_in_tx` : `FOR UPDATE` si `own_fx_accounts` change ; `archive_guarded`, `reactivate` : en partage). La phrase finale reste vraie et reste écrite : ils s'excluent de cette fonction dès son sentinelle.
5. **AC-5.** Vérifié **par grep, sans test** (texte pur) :
   - `grep -nF "prennent \`companies\` en partage d'abord" crates/kesh-db/src/repositories/journal_entries.rs` → **vide** ;
   - `grep -nF "FOR UPDATE\` quand le geste change les comptes en devise propres" crates/kesh-db/src/repositories/journal_entries.rs` → **une** ligne, dans le doc-comment de `create_opening_entry` ;
   - `grep -rnF "en partage d'abord" crates docs` → vide (la fiche 53-3 est hors du périmètre du grep et ne se réécrit pas : elle cite l'issue) ;
   - le diff de `journal_entries.rs` ne porte **que** des lignes `///` (contrôle : `git diff -U0 -- crates/kesh-db/src/repositories/journal_entries.rs | grep -E '^[+-]' | grep -vE '^(\+\+\+|---)' | grep -vE '^[+-]///'` → vide).
6. **AC-6.** Le mode décrit pour **chaque** geste est relu au code et la ligne lue citée au Dev Agent Record (`accounts.rs:1219-1224`, `:215-218`, `:1908`, `:2109`), jamais déduit de l'issue.

### Lot C — #1657 : la garde d'ordre dit sa limite de flux, et la réduit

Choix de l'agent (l'issue offre « une phrase » ou « une garde de flux ») : **les deux, au moindre coût** — la phrase, et une garde lexicale **de niveau de bloc** qui ferme le cas que l'issue décrit (appel sous une branche), sans prétendre à une analyse de flux.

7. **AC-7.** La doc de `la_lecture_des_types_precede_le_recensement` (`opening_types_sous_verrou_lexical.rs:192-199`) ajoute une phrase : la garde compare l'**ordre textuel**, non l'ordre d'exécution ; elle voit un appel déplacé, non un appel rendu conditionnel par une branche ou précédé d'un retour anticipé **sous un autre nom de construction** que ceux qu'elle teste (AC-8).
8. **AC-8.** Un test neuf `la_lecture_des_types_et_le_recensement_sont_au_niveau_du_corps` (même fichier) exige, dans le corps masqué de `create_opening_entry`, pour **chacun** des deux appels (`lock_types_by_ids_in_tx(`, `count_by_company(`) : **exactement une** occurrence, et une **profondeur d'accolades égale à celle de l'ouverture du corps** à sa position (comptage de `{`/`}` sur le texte masqué, de la première accolade du corps jusqu'à l'appel) — donc ni sous un `if`/`match`/boucle/bloc, ni dans une fermeture. Le développeur relit `corps_masque` (`:173-180`) pour savoir si la tranche inclut l'accolade d'ouverture, et fixe la profondeur attendue en conséquence ; il l'écrit en commentaire.
9. **AC-9.** Sondes négatives, sur une **copie du texte réel** (comme `sonde_negative_l_appel_deplace_apres_le_recensement_rougit`, `:210-`), dans le même fichier : (a) l'appel de lecture enveloppé dans `if true { … }` → AC-8 le voit (rouge) ; (b) l'appel dupliqué (deux occurrences) → rouge. Chaque sonde asserte d'abord que sa transformation a bien modifié la copie (ancre unique, comme les sondes existantes) : une sonde dont la substitution ne mord pas serait verte par vacuité.
10. **AC-10.** **Mutations nommées**, appliquées à `create_opening_entry` (`journal_entries.rs`, `let types = super::accounts::lock_types_by_ids_in_tx(…)`), restaurées par `git checkout -- <fichier>` **puis `touch`** (mémoire du dépôt : cargo garde le binaire muté), consignées au Dev Agent Record : **M3** — envelopper l'appel dans `if !account_ids.is_empty() { … }` → AC-8 rougit, `la_lecture_des_types_precede_le_recensement` reste verte (preuve que la garde d'ordre seule ne le voyait pas) ; **M4** — dupliquer la ligne de l'appel → AC-8 rougit.
11. **AC-11.** La doc du test neuf écrit sa propre limite : pas d'analyse de flux (un `return` conditionnel **avant** l'appel, au niveau du corps, lui échappe) ; c'est à la revue de code de le voir.

### Transverse

12. **AC-12.** Aucun fichier de production Rust ne change hors du doc-comment du lot B ; aucun fichier de production frontend ne change ; aucune migration ; aucune clé i18n ; **aucun changement visible de l'utilisateur** (pas de recette visuelle, pas de ligne de manuel à changer — `grep -rn "Remplacer le brouillon\|Ctrl+S" docs/manual` n'est consulté que pour confirmer l'absence d'écart, résultat consigné).
13. **AC-13.** Les gates de clôture tournent sur le **dernier commit de code** : `scripts/test-fast.sh` (fmt + clippy + nextest), `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` ; `gate-ci.sh` puisque `kesh-db` est touché (test et doc-comment d'un module de dépôt) ; E2E complet au dernier commit de code (baseline KF-029). Le Dev Agent Record n'affirme que ce qui a tourné.

## Tasks / Subtasks

- [ ] **Lot A — #1653** (AC: 1, 2, 3)
  - [ ] A.1 Rouge d'abord : écrire l'assertion `defaultPrevented` ; vérifier qu'elle passe sur le code actuel (le défaut est une absence de test, pas un bug).
  - [ ] A.2 Mutations M1 et M2 ; capture de chaque rouge ; restauration ; test vert.
- [ ] **Lot B — #1656** (AC: 4, 5, 6)
  - [ ] B.1 Relire `accounts.rs:1219-1224`, `:215-218`, `:1908`, `:2109` (citer les lignes lues).
  - [ ] B.2 Réécrire la phrase du doc-comment ; greps de AC-5, dont le contrôle « que des `///` ».
- [ ] **Lot C — #1657** (AC: 7 à 11)
  - [ ] C.1 Phrase de limite à la doc de `la_lecture_des_types_precede_le_recensement`.
  - [ ] C.2 Test `…_au_niveau_du_corps` + deux sondes négatives (AC-8, AC-9).
  - [ ] C.3 Mutations M3 et M4 sur le texte réel ; restauration + `touch` ; garde verte.
- [ ] **Clôture** (AC: 12, 13)
  - [ ] Gates de clôture et E2E au dernier commit de code ; Dev Agent Record, Change Log (une entrée) ; `sprint-status.yaml` ; recette de la documentation (aucune).

## Dev Notes

- **Périmètre minimal** (`docs/agents-orchestration.md` § 3) : aucune de ces issues n'élargit. Un défaut voisin découvert se **demande en issue** dans le rapport, il ne s'intègre que s'il est mécanique.
- **Commits** : `refs #1653`, `refs #1656`, `refs #1657` seulement ; aucun mot-clé de fermeture (ils vont dans la PR de l'epic). Un commit par lot.
- **Modules touchés** : `frontend/src/lib/features/journal-entries` (test), `kesh-db` (doc-comment de `repositories/journal_entries.rs` ; test `tests/opening_types_sous_verrou_lexical.rs`). Deux modules, sous la borne de cinq.
- **Réutiliser, ne pas réécrire** : le comptage de profondeur du lot C s'appuie sur `tests/lexique/mod.rs` (`decouper`, `fonctions`, `bloc_apres` `:120`) ; si une fonction de profondeur y existe ou y est naturelle, la mettre là (DRY), sinon une fonction locale de dix lignes. Le texte est **masqué** (chaînes, commentaires, tests exclus) : les accolades d'une chaîne ne faussent pas le compte.
- **Tests Rust** : binaire ciblé `cargo nextest run -E 'binary(opening_types_sous_verrou_lexical)'` (gate ciblé entre deux passes) ; `kesh-db` touché : gate complet au push.
- **Tests frontend** : `scripts/mem-guard.sh npm --prefix frontend run test:unit -- JournalEntryForm.assistant`.
- **Base et ports** : port 3657, bases `kesh_535` et `kesh_e2e_535`, journaux `kesh-gate-logs/e53/s535/`.
- **Manuel** : aucun contrôle visible ni comportement documenté ne change ; `docs/manual/` n'est lu que pour constater l'absence d'écart (AC-12). Le prompt de chaque passe de validation et de revue **nomme** le manuel (CLAUDE.md § *Le prompt d'une passe doit NOMMER le manuel*), PDF aplati compris.
- **Ce qui reste hors de la story** : une garde de **flux** complète (analyse de contrôle) ; non planifiée, la garde de niveau de bloc suffit à la classe décrite. Aucune issue neuve à ouvrir au stade de la fiche.

### Project Structure Notes

- Test frontend : `frontend/src/lib/features/journal-entries/JournalEntryForm.assistant.test.ts` ; test Rust : `crates/kesh-db/tests/opening_types_sous_verrou_lexical.rs` (garde lexicale sur le texte, sans base : à confirmer au développement qu'elle n'est pas à déclarer dans `test_schema_guard.rs`).

### References

- #1653 (53-1a, revue de code P2, L1), #1656 et #1657 (53-3, hors périmètre), #669 (ordre de lecture sous `innodb_snapshot_isolation`), #513.
- `_bmad-output/planning-artifacts/epic-53-ecritures-justes.md` § *Story de solde*.
- `docs/agents-orchestration.md` § 2.1, § 3 ; CLAUDE.md § *Priorités des défauts*, § *Review Iteration Rule*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-28 — Fiche rédigée (Sonnet 5.5). Étape 0 : les trois issues sont encore ouvertes au code ; les numéros de ligne de #1656 étaient périmés (citer ceux d'aujourd'hui). Aucune validation encore.
