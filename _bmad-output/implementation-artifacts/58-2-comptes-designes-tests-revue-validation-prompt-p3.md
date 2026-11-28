# Prompt — validation P3 ciblée (Haiku) de la fiche 58-2 — une seule lentille, sur le seul commit de remédiation de la P2

Worktree `/home/gcorbaz/devel/kesh-58-2`, branche `story/58-2-comptes-designes-tests-revue`. Ne regarde **que** `git diff HEAD~1 HEAD -- _bmad-output/implementation-artifacts/58-2-comptes-designes-tests-revue.md` (diff unique aplati) ; la P2 est dans `/home/gcorbaz/devel/kesh-gate-logs/e58/58-2-validation-p2.md`.
Lentille : **les ajouts de la remédiation sont-ils vrais au code de `90192c7b3` ?** Vérifie chaque affirmation avec `grep -nF` / lecture directe, ligne citée :
1. `services/factures.rs:582` : `lecture.aujourd_hui()` dans `reglements` ; `poignees.rs:62-63` ; autres lignes `aujourd_hui()` énumérées (`factures.rs` 286, 334, 537, 582 ; `factures_fournisseur.rs:206` ; `postes_ouverts.rs:91` ; `rapports.rs` 977, 1025, 1115) ; le décompte des routes (25 en six fichiers : 1+2+6+3+4+9).
2. Épic 53 : `kesh-db/src/repositories/accounts.rs` (+19 dès la l. 458), `kesh-db/src/errors.rs` (+13 dès la l. 570) : `git diff 90192c7b3...origin/epic/53-ecritures-justes`.
3. `accounts.rs:4065` ; forme compilable de la mutation d'AC13(a) (`before_opt` est un `Option<Account>`).
4. Cohérence : statut, Change Log (0/0/1/3), release visée v0.20.1 partout (grep `v0.21.0` : seules les mentions de l'Epic 53 restent).
Axes à déclarer exercés / non exercés. Interdits : lecture seule, aucune commande qui écrit, ni `cargo`/`npm`/`docker`/`scripts/*`, jamais `pkill`/`killall`/`kill` par motif. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e58/58-2-validation-p3.md`, verdict chiffré.
