# Correctif : les quatre suites de la PR #1351 (changement du parent d'un compte)

Status: review

*Fiche rédigée le 2026-11-20, branche `fix/suivis-pr-1437-1440`, partie de `origin/main` (`d29aa557`). Correctif hors
epic, **non urgent** : il ne passe pas par `hotfix/` et va à la prochaine release planifiée. **Aucune migration.**
Quatre points tracés en issues à la revue du correctif `fix-api-parent-compte` (PR #1351), réunis en un seul correctif
(issues #1437, #1438, #1439, #1440).*

## Intention et prémisses vérifiées au code

1. **#1437 — message de `ACCOUNT_PARENT_HAS_ENTRIES`.** `guard_parent_change_in_tx`
   (`crates/kesh-db/src/repositories/accounts.rs`) refuse tout parent qui porte une ligne d'écriture, qu'il soit
   imputable ou non (`has_journal_line_locked`, sans condition sur `postable`). Le texte, lui, disait « y ranger ce
   compte le rendrait non imputable » : faux quand le parent l'est déjà. **Décision** : une seule phrase qui vaut pour
   les deux cas (« aucun compte ne se range sous lui, car son solde ne pourrait plus être corrigé par une saisie »),
   dans les quatre locales et dans le repli de `crates/kesh-api/src/errors.rs` ; le code, les `details` et la règle ne
   changent pas. `docs/api-external.md` (deux passages) suit.
2. **#1438 — E2E `accounts.spec.ts`.** Le bloc « changer le parent d'un compte » créait deux comptes sous un suffixe
   `Date.now()` à quatre chiffres et ne les archivait pas ; le `beforeAll` de la spec reseme la base, ce qui masquait
   le résidu d'une spec à l'autre, non d'une relance sur la même base. Corrigé : suffixe tiré au hasard et contrôlé
   contre les comptes existants (`includeArchived=true`), puis archivage du membre et du groupe dans un `finally`.
3. **#1439 — ordre d'arbre.** `treeAccounts` (`+page.svelte`) calculait le niveau mais gardait l'ordre des numéros.
   Corrigé par `buildAccountTree` (`frontend/src/lib/features/accounts/account-options.ts`) : parcours en profondeur
   depuis les racines (compte sans parent ou dont le parent n'est pas dans la liste), frères triés par numéro, un
   compte pris dans un cycle (aucun chemin d'écriture n'en produit) ajouté à la fin au niveau 0.
4. **#1440 — `account_parent_change.rs`.** La variable `usd` (ligne 445) désignait le compte 1023, **en EUR**, et n'était
   consommée que par `let _ = usd;`. Le compte reste nécessaire (il désigne `g2` avec `eur`) ; la liaison est retirée,
   le commentaire dit pourquoi, et celui de l'assertion suivante nomme « le compte 1023 ». (La variable `usd` de la
   ligne 279 est un autre binding, correct.)

## Critères d'acceptation

- **AC-1** — Le refus d'un parent mouvementé **déjà non imputable** rend un message qui ne dit pas que le parent
  « deviendrait » non imputable (test `put_parent_mouvemente_deja_non_imputable_message_exact`).
- **AC-2** — Un compte rangé sous un groupe dont le numéro ne l'encadre pas s'affiche sous ce groupe (tests Vitest
  `buildAccountTree`, captures avant/après).
- **AC-3** — Le test E2E ne laisse aucun compte actif et ne dépend plus d'un suffixe d'horloge.
- **AC-4** — Aucune variable de test ne porte un nom qui contredit son contenu.
- **AC-5** — Aucune migration ; manuel `.tex` inchangé (aucune phrase ne reprenait le texte du refus).

## Tâches

- [x] Message du refus, quatre locales, repli, `api-external.md`.
- [x] Test d'intégration du message ; mutation (repli de `errors.rs` ramené à l'ancien texte : rouge).
- [x] `buildAccountTree`, `+page.svelte`, quatre tests Vitest ; mutation (ordre des numéros rendu : rouges).
- [x] E2E : numéros libres, nettoyage.
- [x] `usd` → liaison retirée.
- [x] Captures avant/après (`kesh-gate-logs/fix-suivis/captures/`) : 1100 sous 1000 avant, sous le groupe 29 après.
- [ ] Rejeu de l'E2E complet sur la cible cargo propre (voir Change Log).

## Change Log

- 2026-11-20 — Développé. Gate ciblé (`kesh-gate-logs/fix-suivis/cible-*.log`) : fmt, clippy du workspace verts ;
  nextest kesh-db et kesh-api (lib), kesh-i18n, `account_parent_change`, `accounts_e2e`, `textes_coherents`,
  `test_schema_guard`, `i18n_e2e` : 1334 passés, 5 ignorés.

## Notes de version

- **Fixed** — Plan comptable : le refus de ranger un compte sous un parent qui porte des écritures ne dit plus que ce
  parent « deviendrait » non imputable quand il l'est déjà (#1437) ; le plan comptable affiche chaque compte sous son
  vrai parent, même quand les numéros ne l'encadrent pas (#1439).

## Dev Agent Record

### File List

- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/tests/accounts_e2e.rs`
- `crates/kesh-db/tests/account_parent_change.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `docs/api-external.md`
- `frontend/src/lib/features/accounts/account-options.ts`, `account-options.test.ts`,
  `frontend/src/routes/(app)/accounts/+page.svelte`, `frontend/tests/e2e/accounts.spec.ts`
- `_bmad-output/implementation-artifacts/fix-suivis-pr-1437-1440.md`

## Gates (dernier commit de code `fa899ab7`, journaux `kesh-gate-logs/fix-suivis/`)

- Gate ciblé : fmt, clippy workspace, nextest 1334 passés, 5 ignorés (`cible-*.log`).
- Frontend : `check`, `lint-i18n-ownership` verts ; `test:unit` **4631 tests (278 fichiers)** ; `build` vert.
- **Cible cargo** : les gates ci-dessous ont d'abord tourné sur la cible partagée, abandonnée le 2026-11-20 (gates
  faux possibles). Rejoués sur la cible propre `kesh-targets/kesh-wtfixsuivis` (`CARGO_INCREMENTAL=0`) :
  `scripts/test-fast.sh --ci` **6166 passés, 14 ignorés** (`gate-backend.log`, identique au premier passage sur cible
  partagée, `gate-backend-cible-partagee.log`).
- `scripts/gate-ci.sh` (copie `git archive`, cible propre à la copie, donc non concernée) : **6185 passés, 0 échoué,
  18 ignorés** (`gate-ci.log`).
- E2E complet (`LANG=fr_CH.UTF-8`) sur la cible partagée : **478 passés, 10 échecs, 18 ignorés** — les dix attendus
  (sept KF-029, `backup-restore-new-install.spec.ts:262` et `:381`, `opening-balances.spec.ts:156`) ;
  `accounts.spec.ts:292` vert (`e2e.log`). **Rejeu sur la cible propre : voir `e2e-r.log`, à joindre ici** (lancé,
  en attente d'une place de `heavy.sh` à la rédaction).
