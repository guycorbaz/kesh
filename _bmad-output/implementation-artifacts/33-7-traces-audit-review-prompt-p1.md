# Revue de code P1 — 33-7, le journal d'audit (#435, #729, #776), trois lentilles B, E, A (lecture seule)

Tu es **une** des trois lentilles de la passe 1 de `bmad-code-review` de la story **33-7** de Kesh. Contexte frais. Travaille en français soigné. L'orchestrateur te dit laquelle tu es :

- **B — Blind Hunter** : le diff seul, sans la fiche ; cherche ce que le code porte en lui-même (transaction, erreurs avalées, secret qui fuit, écriture provoquée par une route publique, concurrence).
- **E — Edge Case Hunter** : chaque branche et chaque frontière — rejeu dans la grâce, à sa limite, second rejeu, jeton révoqué par déconnexion, révocation à zéro session, trace impossible, clé `read` et `read-write`, clé révoquée ou expirée, huit émetteurs sous session, clé d'API, clé MCP, détail JSON qui n'est pas un objet.
- **A — Acceptance Auditor** : la fiche contre le code, AC par AC, mutation par mutation, manuel `.tex` **et PDF**, `docs/api-external.md`, CHANGELOG, registre.

Kesh 0.17.4 est en production sur le NAS de Guy, avec ses livres réels : note en conséquence.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 log --oneline bdc3faa4..story/33-7-traces-audit
git -C /home/gcorbaz/devel/kesh-e337 diff bdc3faa4 story/33-7-traces-audit -- crates docs CHANGELOG.md
```

⚠️ Le worktree peut porter une **autre** branche au moment de ta lecture : lis toujours par la référence `story/33-7-traces-audit` (`git show story/33-7-traces-audit:<fichier>`), jamais le fichier sur le disque.

Lis le diff **aplati**. Fichiers principaux : `crates/kesh-api/src/routes/auth.rs` (`change_password`, `tracer_vol_de_jeton`, le chemin du rejeu de `refresh`), `crates/kesh-api/src/services/acces.rs` (`OrigineTrace`, `ContexteAcces::mcp`), `crates/kesh-api/src/services/rapports.rs` (`Auteur`, `tracer`, les huit `emit_*`), `crates/kesh-api/src/services/cles_mcp.rs`, `crates/kesh-api/src/audit_labels.rs`, les quatre `crates/kesh-i18n/locales/*/messages.ftl`, et les tests `crates/kesh-api/tests/{auth_audit_e2e,audit_route_registry,services_motif,mcp_outils_e2e}.rs`.

Fiche : `_bmad-output/implementation-artifacts/33-7-traces-audit.md` à la même référence (lentille A ; B la lit **après** son examen à l'aveugle, pour classer). Registre : `e33-choix-autonomes-e334.md`, C-e334-11 à 15, 27, 28, et 32. Décisions de Guy : `_bmad-output/planning-artifacts/epic-33-numerotation-et-defauts.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e337/rouges.log`, `mut/resume.txt`, `mut/M*.log`, `gates-head.log`, `gate-test-fast.log`, `e2e-r1/playwright.log`.

## Axes obligatoires

1. **La transaction de `change_password`** : ce qui reste hors d'elle (révocation, jeton neuf) et ce qu'une panne entre les deux laisse ; le commentaire d'atomicité contre le code.
2. **Le refus d'une clé d'API** : placé avant toute lecture, et le corps JSON invalide (422 avant 403 : voulu ?) ; toute autre route qui rendrait un cookie à une clé.
3. **Le vol de jeton** : `revoked_count > 0`, écriture provoquée par une route publique et ce qui la borne ; attribution au titulaire ; l'échec de l'entrée.
4. **Les sites non résolus** : tout `NewAuditLogEntry::user(` restant sur un chemin qu'une clé d'API atteint dans les rapports (`git grep -n "NewAuditLogEntry::user" crates/kesh-api/src/services/`) ; tout `ContexteAcces::mcp` ; `revoke_all_for_user` (`auth.rs`, `users.rs`, `bootstrap.rs`), chemin par chemin.
5. **`Auteur::detail`** : un détail qui ne serait pas un objet, une clé déjà présente, le JSON exact d'une session inchangé.
6. **La garde `motif_recevable`** : ce qu'elle laisse passer, le motif à l'égalité stricte, les quatre handlers.
7. **Le manuel — `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/user-manual.tex` et leurs PDF** : contrôle chaque ligne du tableau *Manuels* de la fiche, et le PDF aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, sur une copie par `git show`, sans écrire : passe le PDF par l'entrée standard, `git show story/33-7-traces-audit:docs/manual/fr/user-manual.pdf | pdftotext - -`). Le grep de l'AC-9, dans les deux formes. Le manuel dit-il vrai contre le code ? Que dit-il encore d'« exactement comme à l'écran » ?
8. **`docs/api-external.md`** (§ 4, § 7, § 10 : « quatre exceptions », et ce qu'il dit de `PUT /auth/password`), **CHANGELOG** `[0.17.5]`.
9. **Le registre** : `137 + 4 + 3 = 144` et les compteurs, recomptés au fichier.
10. **Les tests** : chacun échoue-t-il pour la raison qu'il nomme ? Le détail `{"revoked_sessions": 2}`, la sonde positive de l'AC-4, le numéro de clé distinct de l'utilisateur.

Aucun contrôle visible neuf : aucune capture n'a été prise, et l'écran du journal se juge par la route de vocabulaire et le manuel.

## Contraintes

⛔ Lecture seule : aucune écriture hors de ton rapport, ni redirection ni fichier temporaire, aucune compilation ni exécution de test ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport : `/home/gcorbaz/devel/kesh-gate-logs/e337-review-p1-{B,E,A}.md`, selon ta lentille. Il porte :

- **les axes réellement exercés et ceux qui ne l'ont pas été**, sans quoi un « 0 finding » ne compte pas ;
- chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait lu, scénario, remédiation ;
- toute affirmation de présence ou d'absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ;
- un décompte recompté.
