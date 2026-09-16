# Story 25.2-a : Le retypage d'un compte mouvementé exige une confirmation explicite

Status: ready-for-dev

**Issues : [#382], [#274].** Les deux décrivent le **même** défaut, au même site, avec le même
correctif — #382 en donne la lecture comptable, #274 le mécanisme et les options. Elles se ferment
ensemble.

## Story

En tant que **comptable tenant les livres dans Kesh**,
je veux que Kesh **refuse de changer le type d'un compte qui porte déjà des écritures tant que je
ne l'ai pas confirmé explicitement**, en me disant combien d'écritures et quels exercices clos
seraient reclassés,
afin qu'**aucun reclassement rétroactif du bilan et du compte de résultat ne soit accidentel**.

## L'arbitrage, et ce qu'il écarte

**Guy, 2026-09-16 : avertissement bloquant.** #274 laissait trois branches ouvertes — *« À trancher :
refus sec vs avertissement bloquant vs reclassement audité »* — et c'est la deuxième qui est retenue.

Ce que cela écarte, et pourquoi il faut l'écrire :

- **le refus sec** — un compte mal typé à la création le resterait à vie, puisque rien ne permet de
  déplacer ses écritures vers un autre compte ;
- **le reclassement audité seul** — journaliser après coup ne rend pas le geste moins muet *au
  moment où il est posé*, et c'est là qu'il fait le dégât.

## Le défaut, établi et non supposé

`accounts::update` (`crates/kesh-db/src/repositories/accounts.rs:360-430`) applique cinq gardes :
compte existant, compte non archivé, verrou optimiste `version`, court-circuit no-op, rôle singleton
non déjà porté par un autre compte. **Aucune ne regarde si le compte porte des écritures**, et
l'`UPDATE` de la ligne 415 écrit `account_type` sans condition.

Or `account_type` est **lu au moment du rapport**, jamais figé à l'écriture :
`crates/kesh-report/src/income_statement.rs:78` (`AND a.account_type = ?`) et
`crates/kesh-report/src/trial_balance.rs:67`.

D'où la conséquence que #382 appelle *« le seul endroit du dépôt où la clôture est contournable sans
réouverture »* : passer un compte de `Expense` à `Asset` fait sortir tout son historique du compte de
résultat pour le faire entrer au bilan — **le résultat d'un exercice clos change**, donc le report à
nouveau, donc le bilan d'ouverture de l'exercice courant. Sans qu'aucune écriture ne l'explique.

⚠️ **Le numéro du compte est déjà immuable, et le rôle est déjà gardé** par la compatibilité
rôle↔type de la 14-3a. C'est bien le seul `account_type` qui est nu.

## Acceptance Criteria

1. **La garde.** `accounts::update` refuse de modifier `account_type` dès que le compte est référencé
   par au moins une ligne d'écriture (`journal_entry_lines.account_id`), sauf confirmation explicite
   de l'appelant. Le refus est un **409** portant le code `ACCOUNT_HAS_ENTRIES`.

2. **Le refus nomme l'ampleur.** Le corps porte `details` avec `entryCount` (nombre d'écritures
   **distinctes**, pas de lignes), `fromType`, `toType`, et `closedFiscalYears` — les noms des
   exercices **clos** touchés, triés, tableau vide si aucun. Un refus qui ne dit pas ce qu'il
   protège ne sert qu'à être contourné.

3. **La garde ne mord que sur le type.** Changer `name`, `role` ou `postable` d'un compte mouvementé
   reste permis sans confirmation. Un payload qui renvoie le **même** `account_type` n'est pas un
   retypage.

4. **Le drapeau.** `UpdateAccountRequest` reçoit `confirmAccountRetype: bool`, `#[serde(default)]`,
   donc **absent ≡ `false`**. Il suit le motif `confirm_*` de
   `crates/kesh-api/src/routes/bank_imports.rs:279-292` — le seul motif de confirmation explicite du
   dépôt, qu'il ne faut pas doubler d'un second.

5. **Confirmé, le geste passe et se distingue dans l'audit.** Un retypage confirmé journalise le code
   d'action **`account.retyped`**, distinct d'`account.updated`, dont les détails portent `fromType`,
   `toType`, `entryCount` et `closedFiscalYears`. Toute autre modification continue de journaliser
   `account.updated` (`accounts.rs:456`).

6. **Le code neuf est inscrit au registre — et cet AC dépend d'une PR non mergée.**
   ⛔ **`crates/kesh-api/src/audit_labels.rs` et `crates/kesh-api/tests/audit_label_registry.rs`
   N'EXISTENT PAS sur `main`** : ils naissent avec la story 25-1c-a, **PR #439, ouverte et non
   mergée** au moment où cette spec est écrite. Vérifié, pas supposé (`git cat-file -e main:…`).
   Deux branches, à trancher **au démarrage du développement** et non maintenant :
   - **#439 mergée d'ici là** → `account.retyped` entre dans `ACTIONS` (92 codes, dont quatre en
     `account.*`) et reçoit sa clé `audit-log-action-account-retyped` dans **les quatre** locales ;
     les tests de `audit_label_registry.rs` doivent rester verts. Sans les quatre traductions, le
     journal d'audit affiche **le code brut** — c'est le repli délibéré de la 25-1c-a, pas un
     accident.
   - **#439 non mergée** → le code d'action s'écrit comme les autres le font sur `main`, en chaîne
     littérale au site d'insertion (`accounts.rs:456` pour le voisin `account.updated`), et
     **l'inscription au registre reste due** : elle est alors portée par la PR qui merge la 25-1c-a,
     ou par une tâche de suivi explicite. Ne pas la laisser tomber en silence — un code absent du
     registre s'affiche brut sans que rien ne rougisse.

7. **Le message est traduit.** `error-account-has-entries` existe dans les quatre
   `crates/kesh-i18n/locales/*/messages.ftl`, sur la convention `error-*` déjà en place.

8. **Un compte vierge n'est pas gêné.** Sans aucune ligne d'écriture, le type se change comme
   aujourd'hui ; le drapeau, présent ou absent, ne change rien.

9. **Le verrou optimiste n'est pas dispensé.** La confirmation ne remplace pas `version`, et le
   conflit de version **précède** la garde de retypage — sinon confirmer un retypage écraserait la
   modification concurrente d'un tiers.

10. **L'écran avertit avant de confirmer.** Dans la boîte de modification
    (`frontend/src/routes/(app)/accounts/+page.svelte`, `submitEdit` ligne 241), un 409
    `ACCOUNT_HAS_ENTRIES` n'est **pas** affiché comme une erreur : il ouvre un avertissement qui
    **nomme** le nombre d'écritures et les exercices clos touchés, et la requête n'est renvoyée avec
    `confirmAccountRetype: true` que sur une action explicite de l'utilisateur. ⚠️ L'écran ne
    pré-vérifie rien : le backend décide, l'écran réagit. Une pré-vérification rouvrirait la fenêtre
    TOCTOU que #274 signale déjà sur `POST /opening-balances`.

11. **Le manuel dit ce qui arrive.** `docs/manual/fr/user-manual.tex`, section du plan comptable :
    le reclassement rétroactif et l'avertissement. PDF régénéré et **contrôlé à plat**
    (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), un `grep` naïf rendant un faux négatif.

12. **Les deux issues se ferment.** La PR porte `closes #382` et `closes #274` ; les commits
    intermédiaires portent `refs`. Le mot-clé va sur la PR, parce que le dépôt merge en squash.

## Tasks / Subtasks

- [ ] **T1 — Dépôt : recenser l'ampleur** (AC 1, 2)
  - [ ] `retype_impact(tx, company_id, account_id) -> Result<(i64, Vec<String>), DbError>` dans
        `accounts.rs` : écritures distinctes + noms des exercices clos touchés, en **une** requête.
  - [ ] Tests : compte vierge → `(0, [])` ; compte mouvementé en exercice ouvert → `(n, [])` ;
        écritures dans un exercice clos → le nom y figure ; deux lignes d'une même écriture ne
        comptent qu'une fois.

- [ ] **T2 — Dépôt : la garde** (AC 1, 3, 8, 9)
  - [ ] `accounts::update` prend `confirm_retype: bool`.
  - [ ] Garde placée **après** le court-circuit no-op (`accounts.rs:395`) et **après** le contrôle de
        version, **avant** l'`UPDATE` — l'ordre est un critère, pas un détail.
  - [ ] `DbError::AccountHasEntries { entry_count, closed_fiscal_years, from_type, to_type }`.
  - [ ] Tests : (type changé / inchangé) × (mouvementé / vierge), plus « nom seul modifié sur compte
        mouvementé », plus « conflit de version sur un retypage confirmé → 409 optimistic lock ».

- [ ] **T3 — API : code, détails, message** (AC 1, 2, 7)
  - [ ] Mapping `AppError` → 409 `ACCOUNT_HAS_ENTRIES`, `details` en camelCase.
  - [ ] `error-account-has-entries` × 4 locales.

- [ ] **T4 — API : le drapeau** (AC 4)
  - [ ] `confirmAccountRetype` dans `UpdateAccountRequest`, `#[serde(default)]`.

- [ ] **T5 — Audit** (AC 5, 6)
  - [ ] `account.retyped` à l'insertion quand le retypage est confirmé.
  - [ ] **Premier geste de la tâche** : constater l'état de la PR #439 (`gh pr view 439`) et
        appliquer la branche correspondante de l'AC 6 — l'inscription au registre n'est possible que
        si `audit_labels.rs` est sur `main`.
  - [ ] Si elle l'est : `ACTIONS` + `audit-log-action-account-retyped` × 4 locales, puis vérifier
        `audit_label_registry.rs` **et** `audit_route_registry.rs`.
  - [ ] Sinon : consigner la dette dans le Change Log **et** la porter à la PR de la 25-1c-a.

- [ ] **T6 — E2E backend** (AC 1, 2, 3, 5, 8)
  - [ ] 409 sans drapeau, corps exact (code + les quatre champs de `details`) ; 200 avec drapeau et
        entrée d'audit `account.retyped` relue en base ; renommage seul non gêné ; compte vierge non
        gêné.

- [ ] **T7 — Frontend** (AC 10)
  - [ ] `accounts.api.ts` : champ optionnel ; `+page.svelte` : rattrapage du 409 et avertissement
        nommant les chiffres ; tests Vitest sur les deux branches (confirmé / abandonné).

- [ ] **T8 — Documentation** (AC 11)
  - [ ] Manuel utilisateur + PDF régénéré ; `docs/api-external.md` ; `CHANGELOG.md`.

- [ ] **T9 — Gates complets et PR** (AC 12)
  - [ ] Base de gate remise à zéro **avant** le gate, inconditionnellement.
  - [ ] Backend, frontend, build, E2E complets ; échecs attendus qualifiés un par un contre
        `docs/testing.md`.

## Dev Notes

### Ce que cette story ne fait pas

Elle ne touche **pas** à la numérotation des écritures (#381) : c'est la story **25-2-b**, et son
arbitrage est encore ouvert. L'epic groupait les deux sous `25-2-gardes-structurelles` ; elles sont
séparées ici parce que leurs sites, leurs risques et leurs décisions n'ont rien de commun.

### Sites exacts

| Quoi | Où |
|---|---|
| l'`UPDATE` nu | `crates/kesh-db/src/repositories/accounts.rs:415-417` |
| les cinq gardes existantes | `accounts.rs:360-412` |
| le court-circuit no-op | `accounts.rs:395` (`is_no_op_change`) |
| l'audit `account.updated` | `accounts.rs:454-457` |
| la route et son contrôle IDOR | `crates/kesh-api/src/routes/accounts.rs:236-252` |
| le motif `confirm_*` de référence | `crates/kesh-api/src/routes/bank_imports.rs:279-292` |
| le registre des libellés d'audit | `crates/kesh-api/src/audit_labels.rs:66-69` |
| la lecture du type par les rapports | `kesh-report/src/income_statement.rs:78`, `trial_balance.rs:67` |
| le formulaire d'édition | `frontend/src/routes/(app)/accounts/+page.svelte:222-252` |

### Contraintes

- ⚠️ **Dépendance à la PR #439, partielle et datée.** Le corps de la story — garde, code d'erreur,
  drapeau, écran, tests — ne dépend de rien et se développe sur `main` tel quel. **Seul l'AC 6**
  (l'inscription du code d'audit au registre) exige `audit_labels.rs`, qui vit uniquement sur la
  branche de la 25-1c-a. Cette spec a d'abord affirmé l'inverse — « la 25-2-a ne dépend pas d'elle » —
  et c'était faux.
- ⛔ **Aucune migration.** La garde est en lecture seule ; rien n'entre au schéma. Les garde-fous
  **P5, P6, P7 et P8** de `CLAUDE.md` sont donc sans objet — et le rester est un critère : si une
  migration apparaît dans le diff, c'est que la story a dérivé.
- **Ne pas déplacer la normalisation `effective_postable`**, qui précède délibérément le
  court-circuit no-op (code review 14-3a, D1/D2).
- La course KF-004 documentée en `accounts.rs:390-394` n'est pas dans le périmètre : la garde de
  retypage ne l'aggrave pas et ne la répare pas.
- Le contrôle IDOR de la route (`find_by_id_in_company`) reste **avant** tout le reste : un 404 ne
  doit jamais devenir un 409 qui révélerait l'existence du compte d'un autre tenant.

### Propagation à greper avant la dernière passe

`account_type` — et non la phrase qui l'entoure — sur tout le dépôt : rapports, bilan d'ouverture,
manuels, `docs/api-external.md`, les quatre `.ftl`. #274 signale nommément que la garde write-time de
`POST /opening-balances` ne peut pas se suffire tant que le retypage est libre : vérifier que son
commentaire dit désormais vrai.

### Règle de splitting

Modules touchés : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — **quatre**, sous le seuil de cinq.
Pas de split préventif.

### References

- [Source: https://github.com/guycorbaz/kesh/issues/382] — lecture comptable, audit du 2026-08-26 § D7.
- [Source: https://github.com/guycorbaz/kesh/issues/274] — mécanisme, corollaire 14-4, les trois options.
- [Source: _bmad-output/planning-artifacts/epic-25-vague1-suite.md:140-147] — § 25-2.
- [Source: CLAUDE.md § Review Iteration Rule, § Migration breaking policy, § Règle de commit et push]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-16 | spec | Story créée. Arbitrage de Guy : **avertissement bloquant** (parmi les trois options de #274). #382 et #274 fusionnées — même site, même correctif. |
