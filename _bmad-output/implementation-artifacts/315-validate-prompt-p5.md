# Story 31-5 — validation P5 ciblée (une lentille, *regression hunter* sur la spec)

*Versionné le 2026-10-24.* **Modèle : Sonnet.** Contexte frais. **Lecture seule stricte.**

## Ce qui est relu

Le seul diff de la remédiation de la validation P4, faite par l'orchestrateur, lu comme **un seul diff aplati** :
`git -C /home/gcorbaz/devel/kesh-31-5 diff 3feaefff 22bae427 -- _bmad-output/`. Il touche la fiche
`_bmad-output/implementation-artifacts/31-5-defauts-courts.md` (*Origine*, Dev Notes sur l'ordre des verrous, Change Log)
et le registre `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-315.md` (C-31-5-16, C-31-5-21). Le rapport
de la P4 qui l'a motivé : `/home/gcorbaz/devel/kesh-gate-logs/315-validate-p4.md`.

## Axes

1. **P4-1** — la nouvelle justification dit qu'aucun appelant de `lock_designated_accounts_in_tx` ne verrouille 1171 et
   2200 ensemble (`SALE` = créance + `VatPayable`, `PURCHASE` = dette + `VatRecoverable`,
   `crates/kesh-db/src/repositories/company_invoice_settings.rs:696-700`). Vérifie-le au code : **tous** les appelants de
   `lock_designated_accounts_in_tx` et les ensembles de rôles qu'ils passent (greper l'appel dans tout `crates/`) ;
   cherche tout autre site qui verrouille deux comptes de TVA ensemble dans un ordre fixe. La phrase « sans effet sur les
   interblocages » est-elle vraie ?
2. **P4-2** — le libellé de #503 dans l'*Origine* et la chaîne 143 → 142 → 138 : cohérents, datés ?
3. **P4-3** — le *Contexte* de C-31-5-16 : l'annotation distingue-t-elle le relevé daté du recompte vivant ?
4. **Résidus** — greper dans la fiche et le registre ce que la remédiation a corrigé (« opposé », « opposition »,
   « fermée par la 31-1c ») : toute occurrence non datée qui affirme encore l'ancien motif est un finding.
5. **Régressions** — la remédiation contredit-elle une autre phrase de la fiche (AC1 l. 95-96, T2, C-31-5-13, C-31-5-18) ?

## Interdits

Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `make`, ni `docker`, ni `curl`, ni `python3`,
ni `sed -i`, ni `scripts/prepare-release.sh`, ni aucun script du dépôt ; aucun `git checkout`/`switch`/`restore`/`reset`/
`stash` ; **aucune redirection vers un fichier, aucun fichier temporaire** ; aucun accès à `~/kesh-test/`. Commandes
permises : `git log`, `git show`, `git diff`, `grep`, `sed -n`, `wc`, `ls`, `gh issue view|list` (lecture), et les
filtres `tr`, `sort`, `cut`, `head`, `awk` vers la sortie standard.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/315-validate-p5.md` — seul fichier écrit. Findings `P5-1…`, sévérité CRITICAL /
HIGH / MEDIUM / LOW, ligne de la fiche ou du registre, site du dépôt, **sortie copiée** (`grep -nF`, `sed -n`) qui
l'établit. Décompte par sévérité **recompté sur la liste**. **Axes exercés et non exercés** : un « 0 finding » sans
cette liste ne compte pas.
