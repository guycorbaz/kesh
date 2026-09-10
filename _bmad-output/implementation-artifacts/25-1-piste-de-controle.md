# Story 25.1 : La piste de contrôle — la rendre inaltérable, puis lisible

Status: ready-for-dev

## Story

**En tant que** responsable des comptes soumis à l'obligation de conservation (CO art. 957-964),
**je veux** que le journal d'audit soit réellement inaltérable et consultable,
**afin que** les corrections rendues *apparentes* par la contre-passation et le gel soient aussi
**imputables** — qu'on puisse dire qui a fait quoi, et qu'aucun geste ordinaire ne l'efface.

⛔ **C'est la contrepartie directe de ce que l'Epic 24 vient de livrer.** La 24-4a a livré la
contre-passation, la 24-4b le gel : une écriture ne se modifie plus, elle se corrige par une
écriture inverse. L'art. 958f CO est satisfait sur la forme. Mais **la seule trace de qui a
corrigé quoi est effaçable par deux chemins et lisible par aucun**. *« Apparent » et « archivé
dans une table que personne ne peut lire » ne sont pas la même chose.*

**Couvre** : [#376], [#377], [#379], [#378] — III.3 de l'audit du 2026-08-26.

## Acceptance Criteria

### Volet A — fermer les deux chemins d'effacement (AVANT tout le reste)

1. **L'import d'une sauvegarde ne détruit plus la piste de contrôle.** `audit_log` sort de
   `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs:55`), ou son traitement à l'import est
   explicitement dissocié du `TRUNCATE`. ⚠️ **Décider ET écrire ce que devient la piste de
   l'instance importatrice** : conservée telle quelle, ou fusionnée avec celle du backup. Les
   deux se défendent ; ce qui ne se défend pas, c'est de ne pas trancher.
2. **La constante sert à deux usages, et il faut le voir avant de la modifier** :
   `admin_backup/export.rs:55` itère `TABLES_TO_TRUNCATE` **pour produire l'export**. En sortir
   `audit_log` sans précaution **retirerait aussi la piste de l'export** — ce qui aggraverait
   [#386], lequel reproche déjà à l'export d'omettre la piste d'audit. ⇒ **l'export DOIT continuer
   de porter `audit_log`.**
3. **`reset_demo` ne supprime plus la piste sans un droit.** Le `DELETE FROM audit_log` non scopé
   de `crates/kesh-seed/src/lib.rs:250` est soit retiré, soit conditionné.
4. **La route `/api/v1/onboarding/reset` exige le rôle administrateur.** Elle est aujourd'hui
   montée dans le bloc « tout rôle authentifié » de `crates/kesh-api/src/lib.rs:773`, **sans
   `require_admin_role`**. ⚠️ **Ses gardes actuelles sont des ÉTATS, pas des droits** —
   `step_completed >= 7`, `!is_demo && step_completed > 2`, et le drapeau d'environnement
   `KESH_PRODUCTION_RESET` (`routes/onboarding.rs:257-277`). *Un état se contourne en amenant le
   système dans l'état voulu ; un droit, non.*
5. **Un test négatif par chemin fermé**, et chacun **éprouvé par mutation** : la garde retirée, le
   test doit rougir. Un test qui passe ne prouve pas qu'il teste.

### Volet B — combler les trous d'alimentation

6. **La gestion des utilisateurs est tracée.** `crates/kesh-api/src/routes/users.rs` porte
   **quatre opérations mutantes et zéro appel au journal** (`grep -c audit_log` rend **0**) :
   `create_user`, `update_user`, `disable_user`, `reset_password`. ⛔ **Un changement de rôle vers
   Comptable ne laisse aujourd'hui aucune trace** — c'est le cas que l'audit cite nommément.
7. **La modification de la société est tracée.** `companies::update`
   (`crates/kesh-db/src/repositories/companies.rs:149-313`) ne trace pas — vérifié, `grep -c` sur
   l'intervalle rend **0**. ⚠️ **Le patron existe DANS LE MÊME FICHIER** : `lock_books` (`:362`)
   et `unlock_books` (`:490`), livrés par la 24-4c, tracent correctement. **Ne pas l'inventer, le
   reprendre.**
8. **Les libellés d'action suivent la convention établie** — `<entité>.<participe>`, relevée au
   sol : `account.created`, `account.updated`, `account.archived`, `books.locked`,
   `invoice.cancelled`… ⇒ `user.created`, `user.updated`, `user.disabled`, `user.password_reset`,
   `company.updated`. **Ne pas introduire une seconde convention.**
9. **Le `details` JSON ne contient aucun secret.** Ni mot de passe, ni empreinte, ni jeton — même
   partiel. ⚠️ Pour `reset_password`, tracer **le fait**, jamais la valeur.

### Volet C — rendre la piste lisible

10. **Une route de consultation existe**, réservée à l'administrateur, filtrable au minimum par
    période, par acteur et par type d'entité, et paginée. Il n'en existe **aucune** aujourd'hui.
11. **Un écran la présente**, en quatre langues, avec les clés i18n dans les **quatre** locales.
12. **La piste reste en lecture seule de bout en bout** : la route ne propose ni suppression, ni
    modification, ni purge. Le repository n'a toujours pas de méthode `delete`.

### Volet D — dire vrai

13. **L'en-tête de `audit_log.rs` est mis à jour.** Il documente aujourd'hui les deux chemins
    d'effacement comme ouverts (correction de la vague 0, issue [#359]) : *« les entrées NE sont
    PAS inamovibles en pratique »*. Une fois les chemins fermés, cette phrase devient fausse **à
    son tour** — dans l'autre sens.
14. **Le manuel est mis à jour, et le PDF régénéré.** ⚠️ **Vérifier ce que le manuel promet
    AUJOURD'HUI de la piste de contrôle avant d'écrire quoi que ce soit** — c'est la leçon la plus
    chère de l'Epic 24 : le manuel y a été pris en défaut à six tours, sous six formes, et c'est
    en vérifiant s'il disait vrai qu'on a trouvé trois chemins d'écriture non énumérés.

## Tasks / Subtasks

- [ ] **T1 — Trancher le sort de la piste à l'import** (AC 1, 2) : lire `backup.rs` et
      `admin_backup/`, établir ce que fait le `TRUNCATE` aujourd'hui, décider *conservation* ou
      *fusion*, **écrire la décision et son motif dans les Dev Notes avant de coder**.
- [ ] **T2 — Fermer le chemin de l'import** (AC 1, 2) + tests, dont un qui **prouve que l'export
      porte toujours `audit_log`**.
- [ ] **T3 — Fermer le chemin `reset_demo`** (AC 3, 4) : garde de rôle sur la route, sort du
      `DELETE`, tests négatifs.
- [ ] **T4 — Éprouver les deux fermetures par mutation** (AC 5).
- [ ] **T5 — Tracer `users.rs`** (AC 6, 8, 9) — quatre opérations.
- [ ] **T6 — Tracer `companies::update`** (AC 7, 8) sur le patron de `lock_books`.
- [ ] **T7 — Route de consultation** (AC 10, 12).
- [ ] **T8 — Écran + i18n ×4** (AC 11).
- [ ] **T9 — En-tête du module et manuel** (AC 13, 14), PDF régénéré.
- [ ] **T10 — Gate complet**, base remise à zéro. ⛔ `kesh-db` est touché : **ciblage interdit**.

⛔ **L'ORDRE N'EST PAS INDIFFÉRENT, et c'est la contrainte centrale de cette story.** T1 à T4
**avant** T7-T8. Rendre la piste *lisible* avant de la rendre *inaltérable* publierait un écran
qui donne à voir une trace qu'un `reset` dément — c'est-à-dire une promesse de plus, du type
exact que la vague 0 a passé treize issues à retirer.

## Dev Notes

### Ce qui a été vérifié au sol pour cette spec, et ce que ça change

**L'en-tête du module ne ment plus — il a été corrigé par la vague 0** ([#359]). Il énonce
désormais les deux chemins d'effacement. *La promesse a été retirée ; le défaut, lui, est intact.*
C'est la story qui doit le fermer, après quoi l'en-tête devra changer une seconde fois (AC 13).

**`audit_log` est bien dans `TABLES_TO_TRUNCATE`** — `backup.rs:55`, entre `journal_entries` et
`api_keys`. ⚠️ **Et la constante a DEUX usages** : `admin_backup/export.rs:55` l'itère pour
produire l'export NDJSON. C'est le piège principal de cette story — le geste évident (« retirer
`audit_log` de la liste ») aggraverait [#386].

**`companies.rs` trace — mais pas là où on croit.** Un `grep -c audit_log` sur le fichier rend un
résultat non nul, ce qui **contredit en apparence** l'affirmation de l'audit. Vérification faite :
les deux insertions sont dans `lock_books` et `unlock_books`, livrés par la 24-4c. La fonction
`update` (`:149-313`) ne trace pas. *L'audit a raison ; un grep au niveau du fichier aurait conclu
l'inverse.*

**`users.rs` ne trace rien** — zéro occurrence, pour quatre opérations mutantes.

**Aucune route de consultation** — zéro résultat sur le montage des routes.

### Le patron de traçage, à reprendre tel quel

```rust
audit_log::insert_in_tx(
    &mut tx,
    NewAuditLogEntry::user(
        user_id,
        "books.locked".to_string(),   // <entité>.<participe>
        "company".to_string(),
        company_id,
        Some(json!({ "before": before, "after": through })),
    ),
)
.await?;
```

⚠️ `insert_in_tx` prend **la transaction en cours** — c'est délibéré : si elle *rollback*, l'entrée
d'audit disparaît avec l'opération auditée. Ne pas la sortir de la transaction pour « garantir »
la trace : on garantirait des traces d'opérations qui n'ont pas eu lieu.

### Ce que la story ne fait pas

- **Aucun trigger SGBD ni `REVOKE`.** L'audit note qu'il n'y en a aucun sur les 61 migrations —
  mais un verrou au niveau base rendrait l'import de sauvegarde et les tests `#[sqlx::test]`
  inopérants. Si le sujet doit être rouvert, c'est une décision d'architecture, pas un détail
  d'implémentation.
- **[#386]** — l'export de souveraineté incomplet est la story **25-5**. Cette story-ci doit
  seulement **ne pas l'aggraver** (AC 2).
- **[#274]** — retyper un compte reclassifie silencieusement : c'est la **25-2**.

### Garde-fous du dépôt qui s'appliquent

- **P5/P6/P7** si une migration est ajoutée — a priori aucune n'est nécessaire, la table existe.
- **Gate complet obligatoire, ciblage interdit** : `kesh-db` est touché (§ *Test Locally First*).
- **Le prompt des passes de revue DOIT nommer le manuel** (§ codifiée le 2026-09-10) — sur une
  story qui touche une promesse publiée, c'est l'axe à plus haut rendement.
- **Un patch vient avec son test**, et le symptôme se grep sur tout le dépôt avant la passe
  suivante — en cherchant **le motif structurel**, pas les valeurs corrigées.

### References

- `audit-experts-2026-08-26.md` § III.3 — *La seule trace de ces corrections est effaçable — et illisible*
- `epic-25-vague1-suite.md` § 25-1
- Issues : [#376], [#377], [#378], [#379] · voisines : [#386] (25-5), [#274] (25-2), [#359] (vague 0, close)
- Sites : `kesh-db/src/backup.rs:55`, `kesh-api/src/admin_backup/export.rs:55`,
  `kesh-seed/src/lib.rs:250`, `kesh-api/src/lib.rs:773`, `kesh-api/src/routes/onboarding.rs:257-289`,
  `kesh-db/src/repositories/companies.rs:149-313` (trou) et `:355-372` (patron),
  `kesh-api/src/routes/users.rs:159-357`, `kesh-db/src/repositories/audit_log.rs:1-21`

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
