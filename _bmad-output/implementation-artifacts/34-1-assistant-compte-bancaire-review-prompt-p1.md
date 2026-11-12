# Prompt — revue de code P1 de la Story 34-1 (#966), trois lentilles (B, E, A)

Tu relis la **Story 34-1** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.6**,
Epic 34 : **l'assistant de démarrage pose la description et le compte comptable lié du compte bancaire**, 1020 présélectionné
à l'écran s'il existe, « modifiable ensuite » écrit sur la page et au manuel. Aucune migration.

Worktree `/home/gcorbaz/devel/kesh-e3401`, branche `story/34-1-assistant-compte-bancaire`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e3401 diff 219bd553 HEAD` (`219bd553` = base ; les commits de comptes rendus en fin de
branche ne portent pas de code). Langue du rapport : français. Passe **complète** (première passe), contexte frais. Modèle :
**Sonnet** ou **Opus** (jamais Haiku pour une première passe).

Une lentille par agent :

- **B — Blind Hunter** : lis le diff **avant** toute autre source ; cherche les défauts du code tel qu'il est écrit.
- **E — Edge Case Hunter** : parcours chaque branche et chaque borne (absent / `null` / valeur ; ordre des refus ; rejeux ;
  compte renuméroté ; liste en échec ; pool d'une connexion ; finalisation après un lien).
- **A — Acceptance Auditor** : AC-1 à AC-9 de la fiche, une par une, contre le code, les tests et le manuel.

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e3401-review-p1-B.md`, `…-E.md`, `…-A.md` (le **seul** fichier que tu écris).

## À lire

- La fiche `_bmad-output/implementation-artifacts/34-1-assistant-compte-bancaire.md` (AC, tâches, mutations, *Ce qui ne se
  traite pas*, **Change Log** et **Dev Agent Record**) ; les registres `e34-choix-autonomes-e341.md` (C-e341-1 à 5, 33, 34) et
  `e34-choix-autonomes-e3401.md` (C-e3401-1 à 7) ; le fichier d'epic `_bmad-output/planning-artifacts/epic-34-reconciliation-comptes-bancaires.md`
  (décision de Guy sur #966).
- Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux (lecture seule) sous
  `/home/gcorbaz/devel/kesh-gate-logs/e3401/` : `rouges.log`, `mut/M*.log` (et `mut/mut-e3401.py`, à **lire**, jamais à
  exécuter), `cible-api-1.log`, `gates-back.txt`, `gate-test-fast.log`, `gate-build.log`, `gate-ci.log`, `gates-front*.txt`,
  `front-*.log`, `e2e-r*.txt`, `e2e-r*.log`, `manuel.log`, `captures-r*.log` (et `captures-e3401.mjs`, à **lire**).
- **Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e3401/captures/34-1-01-etape-1020-preselectionne.png`,
  `34-1-02-etape-saisie.png`, `34-1-03-liste-en-echec.png`. Regarde-les si ton outil lit les images : les deux champs sont-ils
  lisibles, la phrase « se modifient ensuite » visible, le motif d'échec sous le champ ?

⛔ Ne lis rien sous `~/kesh/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier hormis ton rapport. **Aucun fichier temporaire** ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, ni `pkill`, ni `killall`. Aucun `git` qui écrit (`checkout`, `switch`, `restore`, `stash`, `commit`…).
- **Rien qui compile ou exécute** : ni `cargo`, `npm`/`npx`/`node`, `python3`, `perl`, `bash -c`, `make`, `latexmk`, `docker`,
  `mariadb`, `curl`, `playwright`, `vitest`, `claude` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
  `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs`.
- **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` est une
  fonction vers `ugrep`), `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`),
  `wc`, `sort`, `uniq`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Le manuel, `.tex` et PDF aplati, contre le code** : `docs/manual/fr/user-manual.tex` § *Chemin B* (item *Compte
   bancaire*) et § *Comptes bancaires* (« La description se saisit à l'assistant… », la note API « À l'assistant, au
   contraire… ») ; le PDF `docs/manual/fr/user-manual.pdf`, aplati. Chaque phrase est-elle vraie au code ? (« sans le compte
   débiteurs ni le compte créanciers » : l'écran filtre-t-il sur le **rôle** — et le serveur ? ; « présélectionné s'il
   existe » ; « absents, ils restent tels quels ; à `null`, ils sont effacés »). ⚠️ Dans le PDF, l'apostrophe est `’` :
   compte les deux graphies (`pdftotext … | /usr/bin/grep -oF "présélectionné s’il existe" | wc -l`, sous `timeout 60`). Le
   motif de la fiche (`modifi[a-z]* [a-z ]*Comptes bancaires`) ne peut pas correspondre à « se modifient ensuite dans
   Administration → Comptes bancaires » (majuscule, flèche) : contrôle plutôt `-oF "se modifient ensuite"`.
2. **La sémantique absent / `null` / valeur (C-e341-34)** : `BankAccountRequest` (`helpers::double_option`), `PrimaryDetails`,
   `is_no_op_change`, l'`UPDATE` construit (colonnes nommées seulement si présentes ; ordre des `bind`), l'`INSERT`. Un
   `"description": "   "` vaut `null` (effacé) : est-ce le bon sens pour un **rejeu** sur un compte qui en porte une ?
3. **L'ordre des refus et la transaction** : contrôles de forme et `validate_journal_account_id` sur le **pool**, avant la
   transaction (société du jeton) ; dans la transaction, `lock_state_at_step` → `lock_company` → lecture des rôles (sans
   verrou, `RoleLookupLock::None`) → `refuse_if_ledger_is_claim_account` → `upsert_primary_in_tx`. Le refus d'étape est un
   **400** `ONBOARDING_STEP_ALREADY_COMPLETED` (la fiche disait 409 : écart noté au Change Log) — le test le distingue-t-il par
   le code ? La lecture sans verrou est-elle sûre (la finalisation exige l'étape 7, reverrouille en `FOR UPDATE`) ? Un
   interblocage nouveau est-il possible (ordre `onboarding_state → companies → accounts (lecture) → bank_accounts`) ?
4. **La source unique de la lecture par rôle** : `accounts::find_singleton_role_in_tx` réemployée par
   `insert_with_defaults_in_tx` — même SQL au caractère près (`singleton_role = ? ORDER BY id LIMIT 1 FOR UPDATE`), même ordre
   des trois lectures, même comportement sur rôle absent (`Option<i64>` au lieu de `Option<Option<i64>>` + `flatten`) ; un test
   qui attend sur le texte de ces requêtes (processlist) reste-t-il juste ?
5. **L'écran** : `routes/onboarding/+page.svelte` (`$effect` de chargement à l'étape 6, `bankLedgerRequested` non réactif,
   présélection, `bind:value` sur `<option value={null}>`, motif d'échec par `errorMessageOf`, garde
   `onboarding-catch-guard.test.ts`) ; `linkableLedgerAccounts` / `preselectedBankLedgerId` (`format.ts`) et leurs trois
   appelants. **C-e3401-1** : le formulaire *Lier* perd son filtre « numéro commençant par 1 ou 2 » — régression possible
   pour l'utilisateur, ou correction juste (14-3a) ? Que devient l'ordre des options ?
6. **Les témoins et les mutations** : chaque AC a-t-il un test qui rougit sans le code ? Un test peut-il passer **à vide**
   (`waitFor` sur un état déjà vrai ; `findByTestId` d'un élément toujours présent ; simulacre de `fetchAccounts` non lu) ?
   M1 à M7 (M7 rejouée après un premier essai qui ne compilait pas : `mut/M7-essai1-ne-compile-pas.log`) : chacune est-elle
   tuée **par le test attendu** ? Mutations plausibles non jouées à proposer (par exemple : `validate_journal_account_id` avec
   `require_postable = false` ; rôles lus avec `role` au lieu de `singleton_role` ; `description` absente de l'audit de
   création ; présélection gardée après un changement manuel avant la fin du chargement).
7. **Les gardes et compteurs** : `bank-name-inventory.test.ts` (table : onboarding 5, API 3, store retiré), `i18n-keys.test.ts`
   (`sitesTotal` 2922 → 2928), `i18n-un-repli-par-cle.test.ts` (407 → 409) — recompte à la source ; quatre locales des deux
   clés neuves (`onboarding-bank-editable-later`, `onboarding-bank-ledger-load-error`) et repli français identique.
8. **L'audit** : `bank_account.created` porte `description` (forme de `create_bank_account`), les instantanés de
   `bank_account.updated` aussi ; aucun IBAN en clair ; le test 6 d'`onboarding_audit_e2e.rs` mis à jour.
9. **Limites voisines** : l'angle mort `PUT /accounts/{id}` entre l'étape 6 et la finalisation (fiche, *Ce qui ne se traite
   pas*) ; ce que la story aurait dû traiter et ne traite pas. Une issue par problème, sans regroupement.

## Rapport

Pour chaque finding : identifiant (`B-1`, `E-1`, `A-1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier:ligne, preuve
(sortie copiée), correctif proposé. Puis la **liste des axes exercés et des axes non exercés** (un « 0 finding » sans cette
liste ne compte pas), et tout fichier créé par erreur.
