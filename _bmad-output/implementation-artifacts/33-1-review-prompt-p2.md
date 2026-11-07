# Revue de code P2 ciblée — 33-1a et 33-1b (#926), une seule lentille, lecture seule

Tu es la passe 2, **ciblée**, de `bmad-code-review` des stories 33-1a et 33-1b de Kesh. Contexte frais. Travaille en français soigné. Tu ne relis pas les stories : tu relis **la seule remédiation de la passe 1**, là où naissent les défauts de fin de boucle.

⚠️ La migration de ces stories s'exécutera sur les livres réels de Guy. Elle n'est pas modifiée par cet intervalle : vérifie-le.

## Le périmètre, bornes en SHA

```sh
git -C /home/gcorbaz/devel/kesh-e331 log --oneline f1b2df5a..00162556
git -C /home/gcorbaz/devel/kesh-e331 diff f1b2df5a 00162556 -- crates frontend docs CHANGELOG.md _bmad-output/planning-artifacts/architecture.md
```

Le commit de code est `398748cd` ; `b95f82b2` et `00162556` portent les documents et les PDF. Rapports de la passe 1, pour la prémisse : `/home/gcorbaz/devel/kesh-gate-logs/e331-review-p1-{B,E,A}.md` et `e331b-review-p1-{B,E,A}.md`. Comptes rendus : le Dev Agent Record et le Change Log de `_bmad-output/implementation-artifacts/33-1a-numero-interne-fournisseur.md` et `33-1b-numero-interne-ecrans.md`, le registre `e33-choix-autonomes-e331.md`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e331/` (`rouges-p1-front.log`, `mut/resume-p1.log`, `mut/p1-*.log`, `gate-p1.log`, `vitest-p1.log`, `e2e-r5.log`, `e2e-c2.log`, `e2e-c3.log`).

## Les axes

1. **La frappe temporisée** (`supplier-invoices/+page.svelte`) : `beforeNavigate`, `onDestroy`, le `$effect` qui lit `searchTimer`. Un `to` nul (fermeture d'onglet), un lien vers la liste elle-même, le retour arrière : que devient la frappe, et que devient le champ ?
2. **Les blancs de bord coupés** (`non_blank`, `nonBlank`) : quel site lisait la valeur brute, et que change-t-elle ailleurs (libellé d'écriture, audit, refus) ? Rust, TypeScript et le SQL de K8 disent-ils maintenant la même chose ?
3. **Le format** (`validate_numbering_template`, `NUMBERING_SHAPE_RE`) : parité entre les deux, un format déjà enregistré qui ne passerait plus, le préfixe vide, le message rendu.
4. **`from_parts` devenu infaillible** (`routes/payment_batches.rs`) : les quatre routes, ce que voit l'écran, ce que masque le `warn`.
5. **Le piège `ff_numero_interne`** (`kesh-seed/src/pieges.rs`) : l'écriture en SQL, son effet sur les autres tests et sur la fonctionnalité `pieges`, les plafonds de taille mesurés sur cette fixture ; les trois lignes de `mcp_enveloppe_e2e.rs` ; ce que le test nommé prouve de plus.
6. **Les tests renforcés** : (c), le numéro supprimé d'AC20, `COLUMN_DEFAULT`, la confirmation d'un lot. Prouvent-ils ce que leur commentaire dit ? Un autre test de la famille passe-t-il encore par le filet du saut ?
7. **Les textes, `.tex` et PDF** : la brochure, `admin-manual` (le plancher, ce qui échoue sous 10.7, la page MCP, les lignes modifiées), `user-manual` (les lots, la règle de score), le CHANGELOG (entrée retirée, colonnes CSV), `api-external.md`, `ci.md`, `architecture.md`. Chaque affirmation neuve contre le code ; ce qui est laissé comme historique, à bon droit ?
8. **Les comptes rendus** : décomptes recomptés contre les journaux, la phrase sur `60eff6d1`, la répétition inscrite à l'intégration, les findings déclarés corrigés le sont-ils ?

## Lire sans écrire

⛔ Lecture seule : aucune écriture hors de ton rapport, **aucune redirection (`>`) ni fichier temporaire, `mktemp` compris**, aucun `rm`. Trois lentilles de la passe 1 y ont contrevenu pour lire un PDF : c'est inutile, tout se lit par un tube.

```sh
# un fichier versionné, à une révision
git -C /home/gcorbaz/devel/kesh-e331 show 00162556:docs/manual/fr/user-manual.tex | /usr/bin/grep -n "numéro interne"
# un PDF versionné, aplati, par un tube : pdftotext lit l'entrée standard avec `-`
git -C /home/gcorbaz/devel/kesh-e331 show 00162556:docs/manual/fr/marketing-brochure.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oE ".{40}MySQL.{60}"
```

Interdits aussi : toute compilation ou exécution de test, `scripts/prepare-release.sh` et tout script mutant, `git checkout`, `git stash`, `git reset`, `docker`, `~/kesh/`, `~/kesh-test/`, le NAS. `grep` est ici une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`. Pathspec : `crates/*/src/*`, car `crates/*/src` ne désigne rien.

## Le rapport

Écris-le dans `/home/gcorbaz/devel/kesh-gate-logs/e331-review-p2.md`. Il porte :

- **les axes réellement exercés et ceux qui ne l'ont pas été**, sans quoi un « 0 finding » ne compte pas ;
- chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait lu, scénario, remédiation, et s'il vient de la remédiation ou d'avant ;
- toute affirmation de présence ou d'absence adossée à une sortie copiée ;
- un décompte recompté.
