# Prompt de revue de code P4 ciblée — story 37-5

Passe **ciblée**, une lentille, contexte frais, sur la remédiation P3 : `git show 8ab80b05`, worktree
`<worktree>/kesh-s37-5`. Journaux dans `$GATE_LOGS/s37-5/review-p4/`.

À lire : la fiche 37-5 (« Amendement de la revue de code P3 », règle D1, Change Log P1 à P3), la fiche 37-1 (création de
compte, sous-comptes, comptes de différence propres, ordre des verrous de `accounts::create`).

Correctif à relire : `POST /accounts` refuse un sous-compte sous un compte d'actif ou de passif en devise qui porte au
moins une ligne (`409 ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES`, `details { parentAccountId, parentNumber, currency }`),
lignes du parent lues sous son `FOR UPDATE`.

Cherche : défauts introduits — **ordre des verrous** de `accounts::create` (37-1 : `companies` en partage, parent,
désignations…) : la nouvelle lecture des lignes sous verrou forme-t-elle un cycle avec une écriture qui verrouille la
ligne puis le compte (`journal_entries`, contrôle central `LOCK IN SHARE MODE`) ? la route est-elle rejouée sur
interblocage ? ; le refus s'applique-t-il aussi aux autres chemins qui donnent un parent à un compte (`PUT
/accounts/{id}` avec `parentId`, import de plan comptable, restauration, onboarding, seed) — grep — et sinon, le chemin
contourne-t-il la règle ? ; un compte de résultat ou CHF reste-t-il libre ; le message dans les 4 locales ; manuel et
guide d'API contre le code ; les tests rougissent-ils sans le correctif ? Déclare les axes exercés / non exercés.

Phase 2 : corrige, avec tests ; gate complet si `kesh-db` est touché (`$GATE_LOGS/heavy.sh
scripts/test-fast.sh --ci`, sans `mem-guard.sh` en plus), base `kesh_s37_5` sur `kesh-mariadb-dev3` (3308), aucun
`GRANT` ; commit « fix(37-5) : revue de code P4 — … » (aucun mot-clé de fermeture, **aucune mention d'IA**). Change Log
« 2026-11-17 — revue de code P4 ciblée (<modèle>) : <décompte> ». Aucun push. Si tu ne corriges rien : aucun commit,
aucun gate.

Rapport : findings `37-5-CR-P4-N` (sévérité, fichier:ligne, constat, correctif), axes, gates (chiffres), décompte.
