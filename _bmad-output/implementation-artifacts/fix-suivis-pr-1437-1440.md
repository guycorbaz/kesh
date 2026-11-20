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
   les deux cas (« on n'y range pas d'autre compte, car un compte qui a des sous-comptes n'est pas imputable et son
   solde ne se corrige plus par une saisie »), dans les quatre locales et dans le repli de
   `crates/kesh-api/src/errors.rs` ; même correction pour `ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES` ; le code, les
   `details` et la règle ne changent pas. `docs/api-external.md` et `user-manual.tex:1131` suivent.
2. **#1438 — E2E `accounts.spec.ts`.** Le bloc « changer le parent d'un compte » créait deux comptes sous un suffixe
   `Date.now()` à quatre chiffres et ne les archivait pas ; le `beforeAll` de la spec reseme la base, ce qui masquait
   le résidu d'une spec à l'autre, non d'une relance sur la même base. Corrigé : suffixe tiré au hasard et contrôlé
   contre les comptes existants (`includeArchived=true`), puis archivage du membre et du groupe dans un `finally`.
3. **#1439 — ordre d'arbre.** `treeAccounts` (`+page.svelte`) calculait le niveau mais gardait l'ordre des numéros.
   Corrigé par `buildAccountTree` (`frontend/src/lib/features/accounts/account-options.ts`) : parcours en profondeur
   depuis les racines (compte sans parent ou dont le parent n'est pas dans la liste), frères dans l'ordre reçu de l'API (collation SQL, sans retri), un
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
- **AC-5** — Aucune migration ; manuel `.tex` modifié à `user-manual.tex:1131` (phrase du refus `ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES`), PDF non commités (régénérés à la release).

## Tâches

- [x] Message du refus, quatre locales, repli, `api-external.md`.
- [x] Test d'intégration du message ; mutation (repli de `errors.rs` ramené à l'ancien texte : rouge).
- [x] `buildAccountTree`, `+page.svelte`, cinq tests Vitest (dernier état, après la P1) ; la mutation « ordre des numéros rendu » visait l'ancien tri et n'a pas été rejouée sur le code sans tri (le test « ne retrie pas » la couvre par raisonnement).
- [x] E2E : numéros libres, nettoyage.
- [x] `usd` → liaison retirée.
- [x] Captures avant/après (`kesh-gate-logs/fix-suivis/captures/`) : 1100 sous 1000 avant, sous le groupe 29 après.
- [ ] Rejeu de l'E2E complet sur la cible cargo propre (voir Change Log).

## Change Log

- 2026-11-20 — Développé. Gate ciblé (`kesh-gate-logs/fix-suivis/cible-*.log`) : fmt, clippy du workspace verts ;
  nextest kesh-db et kesh-api (lib), kesh-i18n, `account_parent_change`, `accounts_e2e`, `textes_coherents`,
  `test_schema_guard`, `i18n_e2e` : 1334 passés, 5 ignorés.

- 2026-11-20 — Remédiation de la revue P1 (0 CRITICAL, 0 HIGH, 3 MEDIUM, 6 LOW) :
  - M1 : `PR.md` porte un mot-clé par issue (`closes #1437, closes #1438, closes #1439, closes #1440`).
  - M2 : `ACCOUNT_PARENT_HAS_ENTRIES` réécrit au présent, vrai que le parent soit déjà non imputable (bascule de la
    création d'un sous-compte) ou non : « on n'y range pas d'autre compte, car un compte qui a des sous-comptes n'est
    pas imputable et son solde ne se corrige plus par une saisie » (quatre locales, repli de `errors.rs`,
    `api-external.md`, doc-comments de `kesh-db`). Le test monte le cas réel (parent mouvementé, `POST` d'un
    sous-compte qui le bascule, `PUT` d'un autre compte dessous) et compare le message exact.
  - M3 : même correction pour `ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES` (choix : un seul message, vrai dans les deux cas,
    plutôt que deux variantes selon `postable` ; le prédicat du refus ignore `postable`), sites `errors.rs`, quatre
    locales, `api-external.md` (trois endroits), `user-manual.tex:1131`, doc-comments ; test `put_parent_en_devise_…`.
  - L1 : `buildAccountTree` ne trie plus, il garde l'ordre de l'API (collation SQL) ; tests Vitest mis à jour (+1).
  - L2 : création des comptes dans le `try` du nettoyage, `expect.soft` dans le `finally` (n'écrase pas l'erreur d'origine).
  - L3 : les quatre locales sont contrôlées par `textes_coherents` et la garde i18n (clés présentes, mêmes arguments) ;
    le test de message compare le repli de `errors.rs`.
  - L4 : `api-external.md` dit que la règle ne vaut que pour le `PUT` ; L5 : commentaire de `account_parent_change.rs`
    corrigé ; L6 : antécédent levé par la nouvelle rédaction.
  - Réserve E2E : les trois échecs hors KF-029 sont KF-065 (#1282) et KF-067 (#1284), hors du correctif.

- 2026-11-20 — Remédiation de la revue P2 (0 CRITICAL, 0 HIGH, 1 MEDIUM, 5 LOW) :
  - M1 : `PR.md` réécrit pour l'état actuel (nouveau message, jumeau, manuel, `buildAccountTree` sans tri, cinq tests
    Vitest) ; aucun chiffre de gate tant que `chaine4`/`chaine5` ne sont pas finis.
  - L1 : cette fiche propagée (Intention 1 et 3, tâches, AC-5).
  - **À faire après les gates en cours** (fichiers de code, interdits pendant `chaine4`/`chaine5`) : L2, indentation
    du corps du `try` dans `frontend/tests/e2e/accounts.spec.ts` (~290-380) ; L3, commentaire de
    `crates/kesh-db/src/repositories/accounts.rs:2667` (« Le parent passerait non imputable avec son solde », incomplet
    pour le `PUT`) ; L4, apostrophe du message fr en devise (`fr-CH/messages.ftl:1247` et repli `errors.rs:5039` :
    « n'est pas imputable » droite contre « fin d’exercice » typographique, à unifier). Chacun appelle une passe
    ciblée, et le gate complet doit repasser après eux.
  - L5 : assumé (le test compare le repli ; les quatre `.ftl` sont contrôlés par `textes_coherents` et la garde i18n).

## Notes de version

- **Fixed** — Plan comptable : le refus de ranger un compte sous un parent qui porte des écritures ne dit plus que ce
  parent « deviendrait » non imputable quand il l'est déjà, pour un compte en CHF comme en devise (#1437) ; le plan comptable affiche chaque compte sous son
  vrai parent, même quand les numéros ne l'encadrent pas (#1439).

## Dev Agent Record

### File List

- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/tests/accounts_e2e.rs`, `account_currency_e2e.rs`
- `crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/accounts.rs` (doc-comments), `docs/manual/fr/user-manual.tex`
- `crates/kesh-db/tests/account_parent_change.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `docs/api-external.md`
- `frontend/src/lib/features/accounts/account-options.ts`, `account-options.test.ts`,
  `frontend/src/routes/(app)/accounts/+page.svelte`, `frontend/tests/e2e/accounts.spec.ts`
- `_bmad-output/implementation-artifacts/fix-suivis-pr-1437-1440.md`

## Gates

**Après la remédiation de la revue P1 (dernier commit de code `621dfbb6`)** : gate ciblé vert (`cible2-*.log` :
fmt, clippy workspace, nextest 1344 passés, 5 ignorés, dont les deux tests neufs `put_parent_mouvemente_deja_non_imputable_message_exact`
et `put_parent_en_devise_mouvemente_deja_non_imputable_message_exact`). `test-fast --ci`, frontend complet, E2E complet
et `gate-ci.sh` au dernier commit de code : **à compléter** depuis `chaine4.console.log`, `chaine5.console.log` et les
journaux `gate-backend.log`, `front-*.log`, `e2e-r.log`, `gate-ci.log` — rien n'est déclaré vert ici avant lecture.

**Avant la remédiation (commit de code `fa899ab7`, historique)** : `test-fast --ci` 6166 passés, 14 ignorés ;
Vitest 4631 (278 fichiers) ; `gate-ci.sh` 6185 passés, 18 ignorés ; E2E 478 passés, 10 échecs (sept KF-029, trois
KF-065/KF-067 hors correctif), 18 ignorés — sur cible partagée pour l'E2E.
