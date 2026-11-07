# Revue de code P2 ciblée — 33-7, la remédiation de la P1 (#435), une lentille (lecture seule)

Passe ciblée, contexte frais, français soigné. Tu relis **la seule remédiation** de la revue P1 de la story 33-7 de Kesh, non la story entière.

## Le périmètre

```sh
git -C /home/gcorbaz/devel/kesh-e337 diff d02bee8b 78100b5e -- crates docs CHANGELOG.md
```

Bornes en SHA : `d02bee8b` (fin de la P1), `78100b5e` (dernier commit de code). Le worktree porte une autre branche : lis par `git show 78100b5e:<fichier>`, jamais sur le disque. Fiche : `git show story/33-7-traces-audit:_bmad-output/implementation-artifacts/33-7-traces-audit.md`, dernière entrée du Change Log. Findings de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e337-review-p1-{B,E,A}.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e337/p2/` (`rouges.log`, `mut/`, `gates-head.log`, `gate-test-fast.log`).

## Axes obligatoires

1. **`revoke_all_for_user_counting_live`** (`refresh_tokens.rs`) : le compte ne porte-t-il que les sessions vivantes ; les deux `UPDATE` dans leur transaction, l'ordre, `NOW(3)` entre les deux ; une ligne qui échoit entre eux ; un interblocage avec une rotation concurrente ; `reason` écrasé ; ce que voient les autres appelants de `revoke_all_for_user`, un par un.
2. **`change_password`, en `Result` extrait** : le statut et le corps du refus de corps pour une session sont-ils ceux d'avant (`JsonRejection`, contre l'extracteur d'origine) ; `Content-Type` absent ; le registre `audit_route_registry` et les gardes lexicales lisent-ils encore ce handler ; le patron d'`activity`.
3. **`Auteur::detail`** : le `as_object_mut`, et le cas qui n'est pas un objet.
4. **Les tests neufs** : `une_session_echue_n_est_pas_comptee`, `sans_session_vivante_aucune_entree`, `un_rejeu_apres_reconnexion_s_ecrit_de_nouveau`, `une_cle_au_corps_illisible_recoit_le_403` ; chacun échoue-t-il pour la raison nommée (mutations N1 à N4) ; la sonde positive ; le montage à clés leurres de `les_huit_emetteurs_…`.
5. **Le manuel — `docs/manual/fr/admin-manual.tex` (la liste des fermetures aux clés, § du journal d'audit), `docs/manual/fr/user-manual.tex` (§ Traçabilité), et leurs PDF** : contrôle le PDF aplati (`git show 78100b5e:docs/manual/fr/admin-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`). Dit-il vrai : « sessions vivantes », « Kesh observe un rejeu », « au mieux », pas d'écran ; la conduite indiquée (« Mot de passe oublié ») est-elle possible à tout utilisateur ? `docs/api-external.md` § 4, § 7, § 9, § 10 : ce qu'il dit des clés et du corps. CHANGELOG `[0.17.5]`.
6. **Propagation** : `git grep` sur `78100b5e` du symptôme (`mutation via PAT`, `vol d'un jeton`, `réservé à l'interface web`, `revoked_count`), chaque site trié.

## Contraintes

⛔ **La lentille écrit son rapport à ce chemin, et à lui seul : `/home/gcorbaz/devel/kesh-gate-logs/e337-review-p2.md`.** Un rapport rendu par message ne compte pas. Lecture seule hors de ce fichier : ni redirection ni fichier temporaire (les tubes sont permis), aucune compilation ni exécution de test ; `scripts/prepare-release.sh`, `git checkout`, `git stash`, `git switch`, `docker` et tout `gh` qui écrit sont interdits. `grep` est une fonction vers `ugrep` dans ce shell : `/usr/bin/grep` ou `git grep`.

Le rapport porte : **les axes réellement exercés et ceux qui ne l'ont pas été**, sans quoi un « 0 finding » ne compte pas ; chaque finding avec sévérité, fichier et ligne, extrait lu, scénario, remédiation ; toute présence ou absence adossée à une sortie `git grep` ou `/usr/bin/grep -nF` copiée ; un décompte recompté.
