# Revue de code P1 — Story 34-6 (#953) : après un 409, l'édition de la société montre l'écriture concurrente

Passe complète, **trois lentilles** en contexte frais, chacune par un agent distinct (Sonnet ou Opus, jamais Haiku pour
une première passe). Worktree `/home/gcorbaz/devel/kesh-e346`, branche `story/34-6-societe-apres-409`, base `665de151`.
Périmètre : `git diff 665de151..HEAD` (aplati : un seul diff, non la suite des commits).

## Ce que chaque lentille lit d'abord

- La fiche `_bmad-output/implementation-artifacts/34-6-societe-apres-409.md` (Conception, AC-1 à AC-8, mutations M1 à M6),
  son Dev Agent Record et son Change Log.
- Les registres `e34-choix-autonomes-e343.md` (C-e343-6, 7, 14) et `e34-choix-autonomes-e346.md` (C-e346-1 à 7).
- Le `CLAUDE.md` du worktree, § *Review Iteration Rule* (les axes déclarés, le manuel nommé).

## Fichiers du diff

- `frontend/src/lib/features/settings/company-conflict.ts` (la règle, fonction pure) et son test ;
- `frontend/src/lib/features/settings/CompanyConflictNotice.svelte` (l'avis) ;
- `frontend/src/routes/(app)/settings/+page.svelte` (les trois blocs) et `settings-page.test.ts` ;
- `frontend/tests/e2e/company-address.spec.ts`, `frontend/tests/e2e/company-contact-details.spec.ts` ;
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (six clés `settings-company-conflict-*`) ;
- `frontend/src/lib/shared/i18n-keys.test.ts` (compteur `sitesTotal`) ;
- `docs/manual/fr/user-manual.tex` et **`docs/manual/fr/user-manual.pdf`** (§ *Vos coordonnées sur la facture*) ;
- `CHANGELOG.md` (`## [0.17.6] — Non publié`, *Corrigé*).

**Le manuel se contrôle dans le PDF aussi**, aplati avant tout `grep` :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "Si quelqu'un a modifié la société.\{0,900\}"`.
Il dit-il ce que fait le code (quels champs sont montrés, lesquels repris, quand l'édition se ferme, quand le message
« rechargées » reste) ?

**Captures** : `/home/gcorbaz/devel/kesh-gate-logs/e346/captures/` (thème clair). Les ouvrir (outil Read sur l'image) et
dire si l'avis se voit et se lit à côté du champ qu'il nomme.

## Lentille B — Blind Hunter

Diff seul, sans la fiche. Cherche les défauts : états Svelte (`$state`) mal remis à zéro, réactivité perdue (copie de
`addressForm`), `data` remplacé pendant l'envoi, chemins d'erreur (relecture qui échoue, refus 400 après un conflit, double
clic pendant `*Submitting`), typage, textes, testids. Rapport :
`/home/gcorbaz/devel/kesh-gate-logs/e346-review-p1-B.md`.

## Lentille E — Edge Case Hunter

Parcourt chaque branche de `companyEditConflict` et des trois `save*` : `already-saved` sans modification, fusion d'un
champ vidé ailleurs (« (vide) »), pays `null`/minuscule/vide/`CH`, `addressStructured` absent, e-mail avec espaces, second
puis troisième conflit (la copie relue), conflit suivi d'un succès puis d'une réouverture, relecture qui échoue après un
premier avis. Les définitions de la fiche sont la référence : `merged`, `changed`, `already-saved`, `unchanged`. Rapport :
`/home/gcorbaz/devel/kesh-gate-logs/e346-review-p1-E.md`.

## Lentille A — Acceptance Auditor

AC par AC (AC-1 à AC-8, dont 5 bis, 5 ter, 6 bis), et les tâches T1 à T7 : chaque AC a-t-il un test qui rougit sans le
code (les mutations M1 à M6 sont journalisées dans `/home/gcorbaz/devel/kesh-gate-logs/e346/mut/`) ? Les gardes i18n
(AC-7 : quatre locales, repli identique, variables par le troisième argument) ; le manuel (AC-8, PDF) ; le CHANGELOG ; les
décomptes du Dev Agent Record recomptés depuis la source. Rapport :
`/home/gcorbaz/devel/kesh-gate-logs/e346-review-p1-A.md`.

## Règles communes

- **Lecture seule.** Aucune commande qui écrit, compile ou exécute : ni `npm`, ni `npx`, ni `cargo`, ni `make`, ni
  `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `git checkout|switch|reset|commit|stash`. Pas de
  redirection `>`/`>>`, pas de fichier temporaire : le seul fichier écrit est le rapport, par l'outil Write.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `git grep -nF` / `/usr/bin/grep -nF` copiée
  (`grep` du shell est un alias vers `ugrep` : utiliser `/usr/bin/grep`).
- Sévérités CRITICAL / HIGH / MEDIUM / LOW, chacune avec fichier:ligne et preuve.
- **Le rapport déclare les axes exercés ET les axes non exercés.** Un « 0 finding » sans cette liste ne compte pas.
- Aucune donnée réelle (IBAN, nom, montant) ne s'écrit dans le rapport.
